-- Drop legacy wallet tables replaced by entity_wallets and entity_app_wallets.
DROP TABLE IF EXISTS wallets.person_app_wallets;
DROP TABLE IF EXISTS wallets.person_turnkey_refs;
DROP TABLE IF EXISTS wallets.org_wallets;
