CREATE INDEX idx_otp_phone_created ON otp_codes (phone, created_at DESC);
CREATE INDEX idx_otp_phone_expires ON otp_codes (phone, expires_at);
CREATE INDEX idx_users_status_role ON users (status, role);
