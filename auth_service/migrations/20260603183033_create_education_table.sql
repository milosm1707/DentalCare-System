CREATE TABLE education_articles (
                                    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                                    title VARCHAR(255) NOT NULL,
                                    icon VARCHAR(50) NOT NULL DEFAULT 'article',
                                    category VARCHAR(100) NOT NULL,
                                    summary TEXT NOT NULL,
                                    content TEXT NOT NULL,
                                    is_published BOOLEAN NOT NULL DEFAULT true,
                                    created_by UUID REFERENCES users(id),
                                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_education_category ON education_articles(category);
CREATE INDEX idx_education_published ON education_articles(is_published);