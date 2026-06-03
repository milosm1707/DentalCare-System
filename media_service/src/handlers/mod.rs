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
#[derive(serde::Deserialize)]
pub struct PdfRequest {
    pub patient_name: String,
    pub patient_email: String,
    pub dentist_name: String,
    pub dentist_email: String,
    pub clinic_name: String,
    pub clinic_address: String,
    pub appointment_date: String,
    pub start_time: String,
    pub duration: String,
    pub notes: Option<String>,
}

pub async fn upload_file(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
    mut multipart: Multipart,
) -> Result<Json<MediaFile>, AppError> {
    let owner_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;
    let requested_type = params.get("type").cloned().unwrap_or_else(|| "xray".to_string());

    while let Some(field) = multipart.next_field().await.map_err(|_| AppError::InternalError)? {
        let file_name = field.file_name()
            .unwrap_or("unknown")
            .to_string();

        let content_type = field.content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let media_type = match requested_type.as_str() {
            "xray" => MediaType::Xray,
            "profile_image" => MediaType::ProfileImage,
            _ => MediaType::Xray,
        };

        let data = field.bytes().await.map_err(|_| AppError::InternalError)?;
        let file_size = data.len() as i64;

        let unique_name = format!("{}_{}", Uuid::new_v4(), file_name);
        let upload_path = PathBuf::from(&state.config.upload_dir).join(&unique_name);

        fs::create_dir_all(&state.config.upload_dir)
            .await
            .map_err(|_| AppError::FileStorageError)?;

        fs::write(&upload_path, &data)
            .await
            .map_err(|_| AppError::FileStorageError)?;

        let file_path = upload_path.to_string_lossy().to_string();

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
    Json(req): Json<PdfRequest>,
) -> Result<Json<MediaFile>, AppError> {
    let owner_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;
    let upload_dir = state.config.upload_dir.clone();
    let now = chrono::Utc::now();
    let date_str = now.format("%d.%m.%Y").to_string();
    let time_str = now.format("%H:%M").to_string();
    let appointment_id_str = appointment_id.to_string();

    let patient_name = req.patient_name.clone();
    let patient_email = req.patient_email.clone();
    let dentist_name = req.dentist_name.clone();
    let dentist_email = req.dentist_email.clone();
    let clinic_name = req.clinic_name.clone();
    let clinic_address = req.clinic_address.clone();
    let appointment_date = req.appointment_date.clone();
    let start_time = req.start_time.clone();
    let duration_label = if req.duration == "thirty" { "30 minuta" } else { "60 minuta" }.to_string();
    let notes = req.notes.clone().unwrap_or_else(|| "Nema napomene".to_string());

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

        // ========== HEADER ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.add_rect(Rect::new(Mm(0.0), Mm(257.0), Mm(210.0), Mm(297.0)));

        layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
        layer.use_text("DentaCare", 28.0, Mm(15.0), Mm(277.0), &font_bold);
        layer.use_text("Stomatoloska klinika", 12.0, Mm(15.0), Mm(270.0), &font);
        layer.use_text("www.dentacare.rs  |  info@dentacare.rs  |  +381 21 555 000", 9.0, Mm(15.0), Mm(263.0), &font);

        // ========== NASLOV ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.use_text("POTVRDA O ZAKAZANOM TERMINU", 18.0, Mm(15.0), Mm(243.0), &font_bold);
        layer.add_rect(Rect::new(Mm(15.0), Mm(239.5), Mm(195.0), Mm(240.5)));

        layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
        layer.use_text(&format!("Datum izdavanja: {}  |  Vrijeme: {}", date_str, time_str), 9.0, Mm(15.0), Mm(234.0), &font);

        // ========== PODACI O TERMINU ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(0.95, 0.96, 0.98, None)));
        layer.add_rect(Rect::new(Mm(15.0), Mm(185.0), Mm(195.0), Mm(228.0)));

        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.use_text("INFORMACIJE O TERMINU", 11.0, Mm(20.0), Mm(222.0), &font_bold);

        // Datum i vrijeme
        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Datum termina:", 10.0, Mm(20.0), Mm(214.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&appointment_date, 10.0, Mm(75.0), Mm(214.0), &font);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Vrijeme:", 10.0, Mm(20.0), Mm(206.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&start_time, 10.0, Mm(75.0), Mm(206.0), &font);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Trajanje:", 10.0, Mm(20.0), Mm(198.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&duration_label, 10.0, Mm(75.0), Mm(198.0), &font);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Status:", 10.0, Mm(20.0), Mm(190.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.6, 0.3, None)));
        layer.use_text("ZAKAZANO", 10.0, Mm(75.0), Mm(190.0), &font_bold);

        // ========== PODACI O PACIJENTU ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(0.95, 0.96, 0.98, None)));
        layer.add_rect(Rect::new(Mm(15.0), Mm(148.0), Mm(92.0), Mm(180.0)));

        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.use_text("PACIJENT", 11.0, Mm(20.0), Mm(174.0), &font_bold);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Ime i prezime:", 9.0, Mm(20.0), Mm(166.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&patient_name, 9.0, Mm(20.0), Mm(160.0), &font);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Email:", 9.0, Mm(20.0), Mm(154.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&patient_email, 9.0, Mm(20.0), Mm(148.0), &font);

        // ========== PODACI O DOKTORU ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(0.95, 0.96, 0.98, None)));
        layer.add_rect(Rect::new(Mm(118.0), Mm(148.0), Mm(195.0), Mm(180.0)));

        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.use_text("STOMATOLOG", 11.0, Mm(123.0), Mm(174.0), &font_bold);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Ime i prezime:", 9.0, Mm(123.0), Mm(166.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&dentist_name, 9.0, Mm(123.0), Mm(160.0), &font);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Email:", 9.0, Mm(123.0), Mm(154.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&dentist_email, 9.0, Mm(123.0), Mm(148.0), &font);

        // ========== PODACI O AMBULANTI ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(0.95, 0.96, 0.98, None)));
        layer.add_rect(Rect::new(Mm(15.0), Mm(118.0), Mm(195.0), Mm(143.0)));

        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.use_text("AMBULANTA", 11.0, Mm(20.0), Mm(137.0), &font_bold);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Naziv:", 9.0, Mm(20.0), Mm(130.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&clinic_name, 9.0, Mm(55.0), Mm(130.0), &font);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Adresa:", 9.0, Mm(20.0), Mm(122.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&clinic_address, 9.0, Mm(55.0), Mm(122.0), &font);

        // ========== NAPOMENA ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Napomena:", 9.0, Mm(20.0), Mm(108.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text(&notes, 9.0, Mm(55.0), Mm(108.0), &font);

        // ========== UPUTE ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.use_text("UPUTE ZA PACIJENTA", 11.0, Mm(20.0), Mm(95.0), &font_bold);
        layer.add_rect(Rect::new(Mm(15.0), Mm(91.5), Mm(195.0), Mm(92.5)));

        layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
        layer.use_text("1.  Molimo Vas da budete prisutni najmanje 10 minuta prije zakazanog termina.", 9.0, Mm(20.0), Mm(85.0), &font);
        layer.use_text("2.  Ponesite licnu kartu ili pasos.", 9.0, Mm(20.0), Mm(78.0), &font);
        layer.use_text("3.  Ukoliko niste u mogucnosti doci, otkazite termin najmanje 24 sata unaprijed.", 9.0, Mm(20.0), Mm(71.0), &font);
        layer.use_text("4.  Za sve informacije kontaktirajte nas na +381 21 555 000.", 9.0, Mm(20.0), Mm(64.0), &font);

        // ========== ZUTA NAPOMENA ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 0.95, 0.8, None)));
        layer.add_rect(Rect::new(Mm(15.0), Mm(38.0), Mm(195.0), Mm(56.0)));
        layer.set_fill_color(Color::Rgb(Rgb::new(0.8, 0.5, 0.0, None)));
        layer.use_text("NAPOMENA", 10.0, Mm(20.0), Mm(51.0), &font_bold);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.2, 0.0, None)));
        layer.use_text("Ova potvrda je automatski generisana i vazeci je dokaz zakazanog termina.", 9.0, Mm(20.0), Mm(44.0), &font);
        layer.use_text("Cuvajte je do dana termina.", 9.0, Mm(20.0), Mm(39.0), &font);

        // ========== FOOTER ==========
        layer.set_fill_color(Color::Rgb(Rgb::new(0.243, 0.318, 0.710, None)));
        layer.add_rect(Rect::new(Mm(0.0), Mm(20.0), Mm(210.0), Mm(32.0)));
        layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
        layer.use_text("DentaCare 2026  |  Sva prava zadrzana  |  www.dentacare.rs", 8.0, Mm(55.0), Mm(25.0), &font);

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
pub async fn get_patient_files(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    axum::extract::Path(patient_id): axum::extract::Path<Uuid>,
) -> Result<Json<Vec<MediaFile>>, AppError> {
    // Samo stomatolozi i admini mogu vidjeti tuđe fajlove
    if claims.role != "dentist" && claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let files = sqlx::query_as::<_, MediaFile>(
        "SELECT * FROM media_files WHERE owner_id = $1 AND media_type = 'xray' ORDER BY created_at DESC"
    )
        .bind(patient_id)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(files))
}