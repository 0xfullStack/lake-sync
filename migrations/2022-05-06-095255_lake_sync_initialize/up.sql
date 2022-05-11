-- Your SQL goes here

CREATE TABLE "pairs" (
                         "id" serial8,
                         "pair_address" char(42) NOT NULL,
                         "pair_index" bigint NOT NULL,
                         "token0" char(42) NOT NULL,
                         "token1" char(42) NOT NULL,
                         "reserve0" text NOT NULL,
                         "reserve1" text NOT NULL,
                         "factory" char(42) NOT NULL,
                         PRIMARY KEY ("id"),
                         CONSTRAINT "pair_id" UNIQUE ("id")
);

CREATE TABLE "protocols" (
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

ALTER TABLE "pairs" ADD CONSTRAINT "fk_protocol_factory_address" FOREIGN KEY ("factory") REFERENCES "protocols" ("factory_address");

