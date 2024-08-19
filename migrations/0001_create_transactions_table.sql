
CREATE TYPE contract_type AS ENUM (
    'externallyownedaccount',
    'contractaccount',
    'specialcasecontract'
);

CREATE TABLE IF NOT EXISTS transaction (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_hash VARCHAR NOT NULL,
    block_hash VARCHAR NOT NULL,
    from_sender VARCHAR NOT NULL,
    to_reciever VARCHAR NOT NULL,
    tx_value BIGINT NOT NULL,
    mempool_time BIGINT NOT NULL,
    gas BIGINT NOT NULL,
    gas_price BIGINT NOT NULL,
    nonce BIGINT NOT NULL,
    block_number BIGINT NOT NULL,
    contract_type contract_type NOT NULL,
    input TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
