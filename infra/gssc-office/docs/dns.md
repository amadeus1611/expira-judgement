# DNS for gsscph.com

Replace `203.0.113.10` with the server IP. Lower TTLs a day before cutover.

| Type | Name | Value |
|---|---|---|
| A | mail | 203.0.113.10 |
| MX | @ | 10 mail.gsscph.com |
| TXT | @ | `v=spf1 mx -all` (add the relay's include if one is used) |
| TXT | `<selector>._domainkey` | DKIM key from Stalwart admin → Domains → DNS records |
| TXT | _dmarc | `v=DMARC1; p=quarantine; rua=mailto:management@gsscph.com` |
| SRV | _imaps._tcp | 0 1 993 mail.gsscph.com |
| SRV | _submissions._tcp | 0 1 465 mail.gsscph.com |
| CNAME | autoconfig, autodiscover | mail.gsscph.com |
| PTR (set at hosting provider) | 203.0.113.10 | mail.gsscph.com |

Stalwart's admin UI generates the exact record set for the domain; prefer it
over this table when they differ. Remove Microsoft's MX/SPF entries only at cutover.
