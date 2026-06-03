import bcrypt
import psycopg2
import uuid
from faker import Faker
from datetime import datetime, timedelta, date
import random

fake = Faker('en_US')
Faker.seed(42)
random.seed(42)

# Konekcija na baze
auth_conn = psycopg2.connect(
    dbname="auth_db",
    user="postgres",
    password="password",
    host="localhost",
    port=5432
)

appointment_conn = psycopg2.connect(
    dbname="appointment_db",
    user="postgres",
    password="password",
    host="localhost",
    port=5433
)

auth_cur = auth_conn.cursor()
appointment_cur = appointment_conn.cursor()

def hash_password(password):
    return bcrypt.hashpw(password.encode('utf-8'), bcrypt.gensalt()).decode('utf-8')

PASSWORD = hash_password("password123")

print("Brišem postojeće test podatke...")
auth_cur.execute("DELETE FROM refresh_tokens")
auth_cur.execute("DELETE FROM dentist_profiles WHERE user_id IN (SELECT id FROM users WHERE email LIKE '%dentacare.test%')")
auth_cur.execute("DELETE FROM users WHERE email LIKE '%dentacare.test%'")
auth_conn.commit()

appointment_cur.execute("DELETE FROM appointments")
appointment_cur.execute("DELETE FROM available_slots")
appointment_cur.execute("DELETE FROM clinics")
appointment_conn.commit()

print("Kreiram stomatologe...")

SPECIALIZATIONS = [
    "Oralna hirurgija",
    "Ortodoncija",
    "Parodontologija",
    "Dječija stomatologija",
    "Protetika",
    "Endodoncija",
    "Estetska stomatologija",
    "Implantologija"
]

CLINIC_NAMES = [
    "Dental Pro", "Bijeli Osmijeh", "DentaCare Plus", "Oral Centar",
    "Zubna Ambulanta Centar", "Smile Studio", "Denta Vita", "Oral Health",
    "Zubotehnika Novi Sad", "Dental Expert", "Bijeli Zubi", "Stomatolog Plus",
    "Denta Med", "Oral Pro", "Zubni Centar", "Dental Life",
    "Smile Perfect", "Oral Studio", "Denta Care", "Zubna Klinika"
]

STREETS = [
    "Bulevar Oslobođenja", "Bulevar Mihajla Pupina", "Ulica Zmaj Jovina",
    "Dunavska ulica", "Modene", "Futoška ulica", "Hajduk Veljkova",
    "Kisačka ulica", "Laze Telečkog", "Trg Slobode", "Ul. Cara Lazara",
    "Vojvođanska ulica", "Novosadskog sajma", "Jurija Gagarina"
]

dentist_ids = []

for i in range(20):
    dentist_id = str(uuid.uuid4())
    dentist_ids.append(dentist_id)
    first_name = fake.first_name_male() if i % 3 != 0 else fake.first_name_female()
    last_name = fake.last_name()
    email = f"dr.{first_name.lower()}.{last_name.lower()}.{i}@dentacare.test"
    email = email.replace(' ', '').replace('đ','dj').replace('š','s').replace('č','c').replace('ć','c').replace('ž','z')

    auth_cur.execute("""
        INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, is_active, created_at, updated_at)
        VALUES (%s, %s, %s, 'dentist', %s, %s, %s, true, NOW(), NOW())
    """, (dentist_id, email, PASSWORD, first_name, last_name, f"06{random.randint(10000000, 99999999)}"))

    specialization = random.choice(SPECIALIZATIONS)
    auth_cur.execute("""
        INSERT INTO dentist_profiles (id, user_id, specialization, bio, clinic_name, clinic_address, working_hours_start, working_hours_end, created_at)
        VALUES (%s, %s, %s, %s, %s, %s, '08:00:00', '16:00:00', NOW())
    """, (
        str(uuid.uuid4()),
        dentist_id,
        specialization,
        f"Dr. {first_name} {last_name} je specijalist iz oblasti {specialization} sa više od {random.randint(5,25)} godina iskustva.",
        CLINIC_NAMES[i],
        f"{random.choice(STREETS)} {random.randint(1, 150)}, Novi Sad"
    ))

auth_conn.commit()
print(f"  ✓ Kreirano {len(dentist_ids)} stomatologa")

print("Kreiram pacijente...")

patient_ids = []

for i in range(100):
    patient_id = str(uuid.uuid4())
    patient_ids.append(patient_id)
    first_name = fake.first_name()
    last_name = fake.last_name()
    email = f"{first_name.lower()}.{last_name.lower()}.{i}@dentacare.test"
    email = email.replace(' ', '').replace('đ','dj').replace('š','s').replace('č','c').replace('ć','c').replace('ž','z')

    dob = fake.date_of_birth(minimum_age=18, maximum_age=80)

    auth_cur.execute("""
        INSERT INTO users (id, email, password_hash, role, first_name, last_name, phone, date_of_birth, is_active, created_at, updated_at)
        VALUES (%s, %s, %s, 'patient', %s, %s, %s, %s, true, NOW(), NOW())
    """, (patient_id, email, PASSWORD, first_name, last_name, f"06{random.randint(10000000, 99999999)}", dob))

auth_conn.commit()
print(f"  ✓ Kreirano {len(patient_ids)} pacijenata")

print("Kreiram ambulante...")

clinic_ids = []
clinic_dentist_map = {}

for i, dentist_id in enumerate(dentist_ids):
    clinic_id = str(uuid.uuid4())
    clinic_ids.append(clinic_id)
    clinic_dentist_map[clinic_id] = dentist_id

    start_hour = random.choice([7, 8, 9])
    end_hour = start_hour + random.choice([7, 8, 9])

    appointment_cur.execute("""
        INSERT INTO clinics (id, dentist_id, name, address, phone, working_hours_start, working_hours_end, created_at)
        VALUES (%s, %s, %s, %s, %s, %s, %s, NOW())
    """, (
        clinic_id,
        dentist_id,
        CLINIC_NAMES[i],
        f"{random.choice(STREETS)} {random.randint(1, 150)}, Novi Sad",
        f"021{random.randint(100000, 999999)}",
        f"{start_hour:02d}:00:00",
        f"{end_hour:02d}:00:00"
    ))

appointment_conn.commit()
print(f"  ✓ Kreirano {len(clinic_ids)} ambulanti")

print("Kreiram slobodne termine...")

slot_ids = []
today = date.today()
durations = ['thirty', 'sixty']

slot_count = 0
for clinic_id in clinic_ids:
    dentist_id = clinic_dentist_map[clinic_id]

    for day_offset in range(1, 31):
        slot_date = today + timedelta(days=day_offset)

        if slot_date.weekday() >= 5:
            continue

        num_slots = random.randint(3, 8)
        start_times = [f"{h:02d}:00:00" for h in range(8, 17)]
        chosen_times = random.sample(start_times, min(num_slots, len(start_times)))

        for start_time in chosen_times:
            slot_id = str(uuid.uuid4())
            slot_ids.append((slot_id, dentist_id, clinic_id, slot_date, start_time))
            duration = random.choice(durations)

            appointment_cur.execute("""
                INSERT INTO available_slots (id, dentist_id, clinic_id, slot_date, start_time, duration, is_available, created_at)
                VALUES (%s, %s, %s, %s, %s, %s, true, NOW())
            """, (slot_id, dentist_id, clinic_id, slot_date, start_time, duration))
            slot_count += 1

appointment_conn.commit()
print(f"  ✓ Kreirano {slot_count} slobodnih termina")

print("Kreiram zakazane termine...")

NOTES = [
    "Boli me zub s lijeve strane",
    "Potrebna je kontrola",
    "Imam karijese",
    "Bol pri žvakanju",
    "Otekla mi je desna",
    "Slomljen zub",
    "Preventivni pregled",
    "Vadenje zuba",
    "Ugradnja plombe",
    "Čišćenje kamenca",
    None, None, None
]

booked_slots = random.sample(slot_ids, min(80, len(slot_ids)))
appointment_count = 0
statuses = ['scheduled', 'scheduled', 'scheduled', 'confirmed', 'completed', 'cancelled']

for slot_data in booked_slots:
    slot_id, dentist_id, clinic_id, slot_date, start_time = slot_data
    patient_id = random.choice(patient_ids)
    status = random.choice(statuses)
    notes = random.choice(NOTES)
    duration = random.choice(durations)

    appointment_cur.execute("""
        INSERT INTO appointments (id, patient_id, dentist_id, clinic_id, slot_id, status, notes, appointment_date, start_time, duration, created_at, updated_at)
        VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, NOW(), NOW())
    """, (
        str(uuid.uuid4()),
        patient_id,
        dentist_id,
        clinic_id,
        slot_id,
        status,
        notes,
        slot_date,
        start_time,
        duration
    ))

    if status != 'scheduled':
        appointment_cur.execute("""
            UPDATE available_slots SET is_available = false WHERE id = %s
        """, (slot_id,))

    appointment_count += 1

appointment_conn.commit()
print(f"  ✓ Kreirano {appointment_count} zakazanih termina")

# Statistike
auth_cur.execute("SELECT COUNT(*) FROM users WHERE role = 'dentist' AND email LIKE '%dentacare.test%'")
dentist_count = auth_cur.fetchone()[0]

auth_cur.execute("SELECT COUNT(*) FROM users WHERE role = 'patient' AND email LIKE '%dentacare.test%'")
patient_count = auth_cur.fetchone()[0]

appointment_cur.execute("SELECT COUNT(*) FROM clinics")
clinic_count = appointment_cur.fetchone()[0]

appointment_cur.execute("SELECT COUNT(*) FROM available_slots")
slots_total = appointment_cur.fetchone()[0]

appointment_cur.execute("SELECT COUNT(*) FROM appointments")
appointments_total = appointment_cur.fetchone()[0]

print("\n" + "="*50)
print("SEED ZAVRŠEN — STATISTIKE:")
print("="*50)
print(f"  Stomatolozi:      {dentist_count}")
print(f"  Pacijenti:        {patient_count}")
print(f"  Ambulante:        {clinic_count}")
print(f"  Slobodni termini: {slots_total}")
print(f"  Zakazani termini: {appointments_total}")
print("="*50)
print("\nLozinka za sve korisnike: password123")
print("\nPrimjer stomatologa za login:")

auth_cur.execute("SELECT email FROM users WHERE role = 'dentist' AND email LIKE '%dentacare.test%' LIMIT 3")
for row in auth_cur.fetchall():
    print(f"  {row[0]}")

print("\nPrimjer pacijenata za login:")
auth_cur.execute("SELECT email FROM users WHERE role = 'patient' AND email LIKE '%dentacare.test%' LIMIT 3")
for row in auth_cur.fetchall():
    print(f"  {row[0]}")

auth_cur.close()
auth_conn.close()
appointment_cur.close()
appointment_conn.close()
print("\nGotovo!")