CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE appointment_status AS ENUM ('scheduled', 'confirmed', 'cancelled', 'completed');
CREATE TYPE slot_duration AS ENUM ('thirty', 'sixty');

CREATE TABLE clinics (
                         id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                         dentist_id UUID NOT NULL,
                         name VARCHAR(200) NOT NULL,
                         address VARCHAR(500) NOT NULL,
                         phone VARCHAR(20),
                         working_hours_start TIME NOT NULL,
                         working_hours_end TIME NOT NULL,
                         created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE available_slots (
                                 id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                                 dentist_id UUID NOT NULL,
                                 clinic_id UUID NOT NULL REFERENCES clinics(id) ON DELETE CASCADE,
                                 slot_date DATE NOT NULL,
                                 start_time TIME NOT NULL,
                                 duration slot_duration NOT NULL DEFAULT 'thirty',
                                 is_available BOOLEAN NOT NULL DEFAULT true,
                                 created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE appointments (
                              id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                              patient_id UUID NOT NULL,
                              dentist_id UUID NOT NULL,
                              clinic_id UUID NOT NULL REFERENCES clinics(id),
                              slot_id UUID NOT NULL REFERENCES available_slots(id),
                              status appointment_status NOT NULL DEFAULT 'scheduled',
                              notes TEXT,
                              appointment_date DATE NOT NULL,
                              start_time TIME NOT NULL,
                              duration slot_duration NOT NULL,
                              created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                              updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_appointments_patient ON appointments(patient_id);
CREATE INDEX idx_appointments_dentist ON appointments(dentist_id);
CREATE INDEX idx_appointments_date ON appointments(appointment_date);
CREATE INDEX idx_slots_dentist_date ON available_slots(dentist_id, slot_date);