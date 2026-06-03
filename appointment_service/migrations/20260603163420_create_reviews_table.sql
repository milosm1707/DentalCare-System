CREATE TYPE review_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE reviews (
                         id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                         patient_id UUID NOT NULL,
                         dentist_id UUID NOT NULL,
                         appointment_id UUID REFERENCES appointments(id) ON DELETE SET NULL,
                         rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
                         comment TEXT,
                         status review_status NOT NULL DEFAULT 'pending',
                         created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                         updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                         UNIQUE(patient_id, dentist_id, appointment_id)
);

CREATE INDEX idx_reviews_dentist ON reviews(dentist_id);
CREATE INDEX idx_reviews_patient ON reviews(patient_id);
CREATE INDEX idx_reviews_status ON reviews(status);