use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::{errors::AppError, models::{Claims, MediaFile, MediaType}, AppState};

pub async fn upload_file(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Json<MediaFile>, AppError> {
    let owner_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    while let Some(field) = multipart.next_field().await.map_err(|_| AppError::InternalError)? {
        let file_name = field.file_name()
            .unwrap_or("unknown")
            .to_string();

        let content_type = field.content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        // Odredi tip medija
        let media_type = match content_type.as_str() {
            "image/jpeg" | "image/png" | "image/webp" => {
                if file_name.to_lowercase().contains("xray") || file_name.to_lowercase().contains("rtg") {
                    MediaType::Xray
                } else {
                    MediaType::ProfileImage
                }
            }
            "application/pdf" => MediaType::PdfConfirmation,
            _ => return Err(AppError::InvalidFileFormat),
        };

        let data = field.bytes().await.map_err(|_| AppError::InternalError)?;
        let file_size = data.len() as i64;

        // Kreiraj unique naziv fajla
        let unique_name = format!("{}_{}", Uuid::new_v4(), file_name);
        let upload_path = PathBuf::from(&state.config.upload_dir).join(&unique_name);

        // Kreiraj upload direktorijum ako ne postoji
        fs::create_dir_all(&state.config.upload_dir)
            .await
            .map_err(|_| AppError::FileStorageError)?;

        // Sačuvaj fajl
        fs::write(&upload_path, &data)
            .await
            .map_err(|_| AppError::FileStorageError)?;

        let file_path = upload_path.to_string_lossy().to_string();

        // Sačuvaj metadata u bazu
        let media_file = sqlx::query_as::<_, MediaFile>(
            r#"INSERT INTO media_files (owner_id, file_name, file_path, media_type, mime_type, file_size)
               VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
        )
            .bind(owner_id)
            .bind(&file_name)
            .bind(&file_path)
            .bind(&media_type)
            .bind(&content_type)
            .bind(file_size)
            .fetch_one(&state.db)
            .await?;

        return Ok(Json(media_file));
    }

    Err(AppError::InvalidFileFormat)
}

pub async fn get_my_files(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<MediaFile>>, AppError> {
    let owner_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let files = sqlx::query_as::<_, MediaFile>(
        "SELECT * FROM media_files WHERE owner_id = $1 ORDER BY created_at DESC"
    )
        .bind(owner_id)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(files))
}

pub async fn download_file(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(file_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let owner_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let file = sqlx::query_as::<_, MediaFile>(
        "SELECT * FROM media_files WHERE id = $1"
    )
        .bind(file_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    if file.owner_id != owner_id && claims.role != "admin" && claims.role != "dentist" {
        return Err(AppError::Forbidden);
    }

    let data = fs::read(&file.file_path)
        .await
        .map_err(|_| AppError::NotFound)?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &file.mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.file_name),
        )
        .body(Body::from(data))
        .map_err(|_| AppError::InternalError)?;

    Ok(response)
}

pub async fn generate_appointment_pdf(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(appointment_id): Path<Uuid>,
) -> Result<Json<MediaFile>, AppError> {
    let owner_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;
    let upload_dir = state.config.upload_dir.clone();
    let now = chrono::Utc::now();
    let date_str = now.format("%d.%m.%Y").to_string();
    let time_str = now.format("%H:%M").to_string();
    let appointment_id_str = appointment_id.to_string();
    let owner_id_str = owner_id.to_string();

    let (pdf_bytes, file_name, unique_name) = tokio::task::spawn_blocking(move || {
        use printpdf::*;

        let (doc, page1, layer1) = PdfDocument::new(
            "DentaCare Potvrda",
            Mm(210.0),
            Mm(297.0),
            "Layer 1",
        );

        let layer = doc.get_page(page1).get_layer(layer1);
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

        // Header pozadina — plavi pravougaonik
        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.add_rect(Rect::new(Mm(0.0), Mm(257.0), Mm(210.0), Mm(297.0)));

        // Naziv klinike u headeru
        layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
        layer.use_text("DentaCare", 28.0, Mm(15.0), Mm(277.0), &font_bold);
        layer.use_text("Stomatološka klinika", 12.0, Mm(15.0), Mm(270.0), &font);
        layer.use_text("www.dentacare.rs  |  info@dentacare.rs  |  +381 21 555 000", 9.0, Mm(15.0), Mm(263.0), &font);

        // Naslov dokumenta
        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.use_text("POTVRDA O ZAKAZANOM TERMINU", 18.0, Mm(15.0), Mm(243.0), &font_bold);

        // Linija ispod naslova
        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.add_rect(Rect::new(Mm(15.0), Mm(239.0), Mm(195.0), Mm(240.0)));

        // Datum izdavanja
        layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
        layer.use_text(&format!("Datum izdavanja: {}  |  Vrijeme: {}", date_str, time_str), 9.0, Mm(15.0), Mm(234.0), &font);

        // Sekcija: Informacije o terminu
        layer.set_fill_color(Color::Rgb(Rgb::new(0.95, 0.96, 0.98, None)));
        layer.add_rect(Rect::new(Mm(15.0), Mm(190.0), Mm(195.0), Mm(228.0)));

        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.use_text("INFORMACIJE O TERMINU", 11.0, Mm(20.0), Mm(222.0), &font_bold);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("ID Termina:", 10.0, Mm(20.0), Mm(215.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&appointment_id_str, 10.0, Mm(65.0), Mm(215.0), &font);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Status:", 10.0, Mm(20.0), Mm(207.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.6, 0.3, None)));
        layer.use_text("ZAKAZANO", 10.0, Mm(65.0), Mm(207.0), &font_bold);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Pacijent ID:", 10.0, Mm(20.0), Mm(199.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&owner_id_str, 10.0, Mm(65.0), Mm(199.0), &font);

        // Sekcija: Upute
        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.use_text("UPUTE ZA PACIJENTA", 11.0, Mm(20.0), Mm(182.0), &font_bold);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text("1.  Molimo Vas da budete prisutni najmanje 10 minuta prije zakazanog termina.", 10.0, Mm(20.0), Mm(174.0), &font);
        layer.use_text("2.  Ponesite ličnu kartu ili pasoš.", 10.0, Mm(20.0), Mm(166.0), &font);
        layer.use_text("3.  Ukoliko niste u mogućnosti doći, otkažite termin najmanje 24 sata unaprijed.", 10.0, Mm(20.0), Mm(158.0), &font);
        layer.use_text("4.  Za sve informacije kontaktirajte nas na +381 21 555 000.", 10.0, Mm(20.0), Mm(150.0), &font);

        // Sekcija: Napomena
        layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 0.95, 0.8, None)));
        layer.add_rect(Rect::new(Mm(15.0), Mm(125.0), Mm(195.0), Mm(143.0)));
        layer.set_fill_color(Color::Rgb(Rgb::new(0.8, 0.5, 0.0, None)));
        layer.use_text("NAPOMENA", 10.0, Mm(20.0), Mm(138.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.2, 0.0, None)));
        layer.use_text("Ova potvrda je automatski generisana i važi kao dokaz zakazanog termina.", 9.0, Mm(20.0), Mm(131.0), &font);
        layer.use_text("Čuvajte je do dana termina.", 9.0, Mm(20.0), Mm(127.0), &font);

        // Footer linija
        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.add_rect(Rect::new(Mm(0.0), Mm(15.0), Mm(210.0), Mm(17.0)));

        layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
        layer.use_text("DentaCare © 2026  |  Sva prava zadržana  |  www.dentacare.rs", 8.0, Mm(55.0), Mm(15.5), &font);

        let pdf_bytes = doc.save_to_bytes()?;
        let file_name = format!("potvrda_{}.pdf", appointment_id_str);
        let unique_name = format!("{}_{}", Uuid::new_v4(), file_name);

        Ok::<_, printpdf::Error>((pdf_bytes, file_name, unique_name))
    })
        .await
        .map_err(|_| AppError::InternalError)?
        .map_err(|_| AppError::InternalError)?;

    let file_size = pdf_bytes.len() as i64;
    let upload_path = PathBuf::from(&upload_dir).join(&unique_name);

    fs::create_dir_all(&upload_dir)
        .await
        .map_err(|_| AppError::FileStorageError)?;

    fs::write(&upload_path, &pdf_bytes)
        .await
        .map_err(|_| AppError::FileStorageError)?;

    let file_path = upload_path.to_string_lossy().to_string();

    let media_file = sqlx::query_as::<_, MediaFile>(
        r#"INSERT INTO media_files
           (owner_id, file_name, file_path, media_type, mime_type, file_size, appointment_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"#,
    )
        .bind(owner_id)
        .bind(&file_name)
        .bind(&file_path)
        .bind(MediaType::PdfConfirmation)
        .bind("application/pdf")
        .bind(file_size)
        .bind(appointment_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(media_file))
}