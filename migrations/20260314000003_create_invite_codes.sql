CREATE TYPE invite_code_status AS ENUM ('unused', 'used', 'revoked');

CREATE TABLE invite_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT NOT NULL UNIQUE,
    distributor_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    status invite_code_status NOT NULL DEFAULT 'unused',
    redeemer_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    redeemed_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    CHECK ((status = 'used') = (redeemer_id IS NOT NULL)),
    CHECK ((status = 'used') = (redeemed_at IS NOT NULL)),
    CHECK ((status = 'revoked') = (revoked_at IS NOT NULL))
);

CREATE INDEX idx_invite_codes_distributor ON invite_codes(distributor_id);
CREATE INDEX idx_invite_codes_status ON invite_codes(status) WHERE status = 'unused';
