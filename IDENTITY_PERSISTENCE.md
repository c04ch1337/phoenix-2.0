# Identity Persistence, Evolution, and Multi-User Support

This repo persists identity state into the **Soul Vault** (via `VitalOrganVaults::store_soul()` / `recall_soul()`) so that runtime identity changes survive restarts.

## Soul Vault Keys

### Phoenix

- `phoenix:preferred_name` — Phoenix's preferred name (primary key)
- `phoenix:name` — legacy compatibility key (still written)
- `phoenix:evolution_history` — JSON array of evolution entries

### Users (multi-user)

Per-user keys are namespaced by user UUID:

- `user:{USER_ID}:name`
- `user:{USER_ID}:preferred_alias`
- `user:{USER_ID}:relationship`
- `user:{USER_ID}:evolution_history` — JSON array of evolution entries

Global default keys are still supported for backward compatibility (primary user):

- `user:preferred_alias`
- `user:relationship`
- `user:evolution_history`

## Primary user

The **primary user** is represented by the nil UUID (`00000000-0000-0000-0000-000000000000`).

This gives a stable ID across restarts while remaining compatible with the older global keys.

## Evolution history

Identity updates append an evolution log entry (timestamp, change type, reason, field, old value, new value).

History is persisted as JSON under the `*:evolution_history` keys.

## Optional autonomous evolution

Phoenix includes a conservative `self_evolve` hook that only runs when explicitly enabled.

Set:

- `PHOENIX_SELF_EVOLVE_SUGGESTED_NAME="Eternal Flame"`

and then call the `CerebrumNexus::evolve_identities()` hook to persist the change.

