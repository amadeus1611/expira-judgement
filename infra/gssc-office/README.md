# GSSC office stack (gsscph.com)

Self-hosted replacement for Outlook and OneDrive/Google Drive, Rust-first.
Lives here until it gets its own private repo (`gssc-infra`); it shares nothing
with the EXPIRA Judgement workspace.

| Part | Software (language) | Replaces |
|---|---|---|
| Mail, calendar, contacts, files (WebDAV) | Stalwart v0.15.5 (Rust) | Outlook / Exchange, OneDrive basic |
| Storage pool behind it | Garage v2.3.0 (Rust, S3) | cloud storage quota |

## On storage limits ("unlimited")
Research (2026-10-18): no mainstream provider offers free unlimited storage;
free tiers are small and unlimited plans are paid. The practical route
is to own the disks: Garage pools storage across any number of disks and
machines (old PCs, a NAS, a free cloud VM) and grows by adding nodes, so the
ceiling is the hardware you add, not a vendor quota. Keep one off-site node
before relying on it (replication_factor 3 needs 3 nodes).

## Tested in the build container (2026-10-18)
- Both containers start; Garage layout, bucket and key created by `scripts/garage-init.sh`.
- Domain `gsscph.com` and user `management` created via the Stalwart API.
- SMTP on 25 accepts mail; a test message to management@gsscph.com was delivered locally.
- WebDAV file upload (201) and download as `management` work.
- Not tested: internet delivery, TLS certificates, web admin UI (its bundle
  downloads from GitHub on first start and was blocked in the sandbox), and
  Stalwart → Garage blob storage wiring.

## Deploy
1. Server with a public static IP, reverse DNS (PTR) = `mail.gsscph.com`,
   and outbound port 25 allowed (Oracle/Azure block it by default — request an
   exemption or set an outbound relay on 587 in Stalwart).
2. `./scripts/gen-secrets.sh && docker compose up -d`
3. `docker compose logs stalwart` prints the admin password; open
   `http://<server>:8080`, set hostname `mail.gsscph.com`, enable ACME TLS,
   then remove the 8080 port mapping.
4. `./scripts/garage-init.sh <capacity>`; put the key into Stalwart's S3 blob store.
5. Create the domain and users; publish DNS from `docs/dns.md`.
6. Migrate with `docs/migrate.md`.

If the host has no IPv6, change Stalwart's generated `bind = "[::]:…"`
lines in `data/stalwart/etc/config.toml` to `0.0.0.0:…`.

## Clients
- Mail: Thunderbird, phone mail apps, or Outlook desktop (IMAP).
- Calendar/contacts: CalDAV/CardDAV at `https://mail.gsscph.com/dav/`.
- Files: WebDAV at `https://mail.gsscph.com/dav/file/<user>/` (Windows map
  drive, macOS Finder, `rclone mount`).

Sources: Stalwart docs https://stalw.art · Garage https://garagehq.deuxfleurs.fr ·
Oracle SMTP block https://docs.oracle.com/en-us/iaas/releasenotes/changes/f7e95770-9844-43db-916c-6ccbaf2cfe24 ·
free storage survey https://gizmodo.com/best-cloud-storage/free
