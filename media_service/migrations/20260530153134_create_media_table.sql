CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE media_type AS ENUM ('xray', 'profile_image', 'chat_image', 'pdf_confirmation');

CREATE TABLE media_files (
                             id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                             owner_id UUID NOT NULL,
                             file_name VARCHAR(255) NOT NULL,
                             file_path VARCHAR(500) NOT NULL,
                             media_type media_type NOT NULL,
                             mime_type VARCHAR(100) NOT NULL,
                             file_size BIGINT NOT NULL,
                             appointment_id UUID,
                             created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_media_owner ON media_files(owner_id);
CREATE INDEX idx_media_appointment ON media_files(appointment_id);
CREATE INDEX idx_media_type ON media_files(media_type);