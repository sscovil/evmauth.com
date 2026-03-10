-- Authorization codes: short-lived single-use codes for the end-user
-- PKCE token exchange flow (code -> JWT).
-- Codes are stored hashed; the plaintext is returned once to the redirect URI.
CREATE TABLE auth.auth_codes (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code_hash               TEXT NOT NULL UNIQUE,
    app_registration_id     UUID NOT NULL,
    person_app_wallet_id    UUID NOT NULL,
    code_challenge          TEXT NOT NULL,
    redirect_uri            TEXT NOT NULL,
    state                   TEXT NOT NULL,
    expires_at              TIMESTAMPTZ NOT NULL,
    used_at                 TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_auth_codes_code_hash ON auth.auth_codes(code_hash);
CREATE INDEX idx_auth_codes_expires_at ON auth.auth_codes(expires_at);
