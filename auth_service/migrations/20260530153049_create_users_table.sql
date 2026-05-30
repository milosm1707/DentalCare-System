CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_role AS ENUM ('patient', 'dentist', 'admin');

CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                       email VARCHAR(255) UNIQUE NOT NULL,
                       password_hash VARCHAR(255) NOT NULL,
                       role user_role NOT NULL DEFAULT 'patient',
                       first_name VARCHAR(100) NOT NULL,
                       last_name VARCHAR(100) NOT NULL,
                       phone VARCHAR(20),
                       date_of_birth DATE,
                       profile_image_url VARCHAR(500),
                       is_active BOOLEAN NOT NULL DEFAULT true,
                       created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                       updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE dentist_profiles (
                                  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                                  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                  specialization VARCHAR(200),
                                  bio TEXT,
                                  clinic_name VARCHAR(200),
                                  clinic_address VARCHAR(500),
                                  working_hours_start TIME,
                                  working_hours_end TIME,
                                  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE refresh_tokens (
                                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                token VARCHAR(500) NOT NULL UNIQUE,
                                expires_at TIMESTAMPTZ NOT NULL,
                                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_refresh_tokens_token ON refresh_tokens(token);
CREATE INDEX idx_dentist_profiles_user_id ON dentist_profiles(user_id);