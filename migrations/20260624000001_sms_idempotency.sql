-- Track season-open SMS notifications per user per season.
-- Separate table because the notification fires before enrollment exists.
CREATE TABLE season_open_notifications (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    season_id UUID NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    notified_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, season_id)
);

-- Track confirm-ready nudge SMS per enrollment.
ALTER TABLE enrollments
    ADD COLUMN confirm_nudge_sent_at TIMESTAMPTZ;

-- Track receipt nudge SMS per assignment.
ALTER TABLE assignments
    ADD COLUMN receipt_nudge_sent_at TIMESTAMPTZ;
