# Migration off Microsoft 365

1. Create every mailbox in Stalwart first.
2. Copy mail while Microsoft still receives it (repeatable, incremental):
   `imapsync --host1 outlook.office365.com --user1 management@gsscph.com --oauthaccesstoken1 <token>
    --host2 mail.gsscph.com --user2 management --password2 <pw> --ssl1 --ssl2`
3. Copy OneDrive files with rclone (configure remotes `onedrive` and `gssc`
   — the latter type `webdav`, vendor `other`, URL `https://mail.gsscph.com/dav/file/management/`):
   `rclone sync onedrive: gssc: --progress`
4. Export calendar (.ics) and contacts (.vcf) from Outlook; import via any CalDAV/CardDAV client.
5. Cutover: switch MX/SPF/DKIM per `dns.md`, run imapsync once more, keep
   Microsoft active two to four weeks, then cancel.
