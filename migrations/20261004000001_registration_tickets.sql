-- Server-side proof that a phone passed OTP verification and may register.
-- The client holds only a random token; the phone is never read from the client.
CREATE TABLE registration_tickets (
    token_hash TEXT PRIMARY KEY,
    phone TEXT NOT NULL,
    invite_attempts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);
