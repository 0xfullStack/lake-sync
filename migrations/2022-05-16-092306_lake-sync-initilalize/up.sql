-- Your SQL goes here


CREATE TABLE "Protocol" (
    "id" serial8,
    "name" varchar(255) NOT NULL,
    "official_url" varchar(255),
    "network" varchar(255) NOT NULL,
    "description" text,
    "symbol" varchar(255),
    "router_address" char(42) NOT NULL,
    "factory_address" char(42) NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "protocol_id" UNIQUE ("id"),
    CONSTRAINT "protocol_factory_address" UNIQUE ("factory_address")
);

CREATE TABLE "Pair" (
    "id" serial8,
    "pair_address" char(42) NOT NULL,
    "factory_address" char(42) NOT NULL,
    "token0" char(42) NOT NULL,
    "token1" char(42) NOT NULL,
    "block_number" bigint NOT NULL,
    "block_hash" text NOT NULL,
    "transaction_hash" text NOT NULL,
    "reserve0" text NOT NULL,
    "reserve1" text NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "pair_id" UNIQUE ("id"),
    CONSTRAINT "pair_address" UNIQUE ("pair_address")
);

CREATE TABLE "ReserveLog" (
    "id" serial8,
    "pair_address" char(42) NOT NULL,
    "reserve0" text NOT NULL,
    "reserve1" text NOT NULL,
    "block_number" bigint NOT NULL,
    "block_hash" text NOT NULL,
    "transaction_hash" text NOT NULL,
    PRIMARY KEY ("id"),
    CONSTRAINT "reserve_log_id" UNIQUE ("id")
);

ALTER TABLE "Pair" ADD CONSTRAINT "fk_protocol_factory_address" FOREIGN KEY ("factory_address") REFERENCES "Protocol" ("factory_address");
-- ALTER TABLE "ReserveLog" ADD CONSTRAINT "fk_pair_pair_address" FOREIGN KEY ("pair_address") REFERENCES "Pair" ("pair_address");

CREATE INDEX "index_pair_block_number" ON "Pair" ("block_number");
CREATE INDEX "index_pair_factory_address" ON "Pair" ("factory_address");
CREATE UNIQUE INDEX "index_pair_pair_address" ON "Pair" ("pair_address");

CREATE INDEX "index_reservelog_block_number" ON "ReserveLog" ("block_number");
CREATE INDEX "index_reservelog_pair_address" ON "ReserveLog" ("pair_address");


