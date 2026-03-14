CREATE TYPE season_phase AS ENUM (
    'enrollment',
    'preparation',
    'assignment',
    'delivery',
    'complete',
    'cancelled'
);

CREATE TYPE user_role AS ENUM (
    'participant',
    'admin'
);

CREATE TYPE user_status AS ENUM (
    'active',
    'deactivated'
);

CREATE TYPE receipt_status AS ENUM (
    'no_response',
    'received',
    'not_received'
);
