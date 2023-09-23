# Library for concurrent requests to the Steam-API

## TODO

- Maybe disabling cookies ups the requests per second before getting 429
- Implement `ClanId` similar to `SteamId`
- Implement `PersonaStateFlags`
  - <https://docs.rs/bitflags/latest/bitflags/>
  - <https://steam.readthedocs.io/en/latest/api/steam.enums.html#steam.enums.common.EPersonaStateFlag>
  - <https://github.com/SteamRE/SteamKit/blob/0d5d7c052602a824d163064c39a9d7f7c12ba6c4/Resources/SteamLanguage/enums.steamd#L196>
