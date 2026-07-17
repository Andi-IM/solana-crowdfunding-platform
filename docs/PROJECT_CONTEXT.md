# Solana Crowdfunding Platform - Project Context

## Status Dokumen

Dokumen ini adalah sumber konteks awal sebelum implementasi program dilakukan. Tujuannya menjaga arah aplikasi agar tetap fokus pada crowdfunding berbasis escrow di Solana: dana donor dikunci sampai campaign memenuhi kondisi sukses atau gagal.

Belum ada keputusan final untuk nama proyek. Nama kerja sementara:

**VaultRaise**

Alasan: nama ini menekankan dua nilai utama produk, yaitu dana masuk ke vault yang terkunci dan hanya dapat dicairkan ketika aturan campaign terpenuhi.

Alternatif branding untuk dipertimbangkan:

- **GoalVault**: jelas dan langsung menjelaskan dana dikunci sampai goal terpenuhi.
- **PledgeLock**: menekankan pledge/donasi yang belum langsung diterima creator.
- **CrowdVault**: sederhana, cocok untuk platform crowdfunding umum.
- **MilestoneVault**: cocok jika nanti berkembang ke campaign berbasis milestone.
- **TrustRaise**: menonjolkan trust dan transparansi untuk donor.

Untuk fase MVP, gunakan nama kerja **VaultRaise** di dokumentasi dan komentar internal sampai nama final dipilih.

## Problem Statement

Crowdfunding tradisional sering memiliki masalah kepercayaan:

- Donor ingin berdonasi tanpa dana langsung diterima creator sebelum syarat campaign terpenuhi.
- Creator perlu mekanisme klaim yang jelas jika target dana tercapai.
- Donor perlu refund otomatis atau dapat diverifikasi jika target tidak tercapai.
- Semua pihak perlu bukti on-chain bahwa dana terkunci sampai kondisi campaign terpenuhi.

Platform ini menyelesaikan masalah tersebut dengan Solana Program yang menyimpan dana kontribusi di vault PDA, bukan langsung ke wallet creator.

## Tujuan MVP

MVP hanya mencakup empat aksi utama:

1. Membuat campaign.
2. Memberikan kontribusi SOL ke campaign.
3. Creator menarik dana jika campaign sukses.
4. Donor mengambil refund jika campaign gagal.

Tidak termasuk dalam MVP:

- Token SPL.
- Campaign bertahap atau milestone.
- Voting donor.
- Biaya platform.
- Frontend lengkap.
- KYC atau identitas legal creator.
- Moderasi campaign.

## Terminologi

- **Campaign**: Data crowdfunding yang dibuat oleh creator.
- **Creator**: Wallet yang membuat campaign dan berhak withdraw jika campaign sukses.
- **Donor**: Wallet yang memberikan kontribusi ke campaign.
- **Vault**: PDA milik program yang menyimpan SOL kontribusi.
- **Contribution**: Data kontribusi per donor per campaign.
- **Goal**: Target dana campaign dalam lamports.
- **Deadline**: Unix timestamp saat campaign berakhir.
- **Raised**: Total lamports yang sudah dikontribusikan.
- **Claimed**: Penanda bahwa dana campaign sukses sudah ditarik creator.

## Model Akun

### Campaign Account

Menyimpan state utama campaign.

Field awal:

```text
creator: Pubkey
goal: u64
raised: u64
deadline: i64
claimed: bool
bump: u8
vault_bump: u8
```

Catatan:

- `goal` disimpan dalam lamports.
- `deadline` menggunakan unix timestamp dengan tipe `i64`, mengikuti `Clock::unix_timestamp` dari Solana.
- `raised` dimulai dari `0`.
- `claimed` dimulai dari `false`.
- `bump` dan `vault_bump` disimpan agar PDA signing lebih eksplisit.

### Vault PDA

Vault adalah PDA yang menyimpan SOL campaign.

Vault tidak perlu menyimpan data kompleks. Opsi implementasi:

- System account PDA dengan lamports saja.
- PDA turunan dari campaign.

Seed yang disarankan:

```text
vault = ["vault", campaign.key()]
```

### Contribution Account

Contribution dibutuhkan agar refund dapat dilakukan dengan benar per donor.

Field awal:

```text
campaign: Pubkey
donor: Pubkey
amount: u64
refunded: bool
bump: u8
```

Seed yang disarankan:

```text
contribution = ["contribution", campaign.key(), donor.key()]
```

Alasan akun contribution diperlukan:

- Program harus tahu berapa jumlah refund donor.
- Program harus mencegah refund ganda.
- Program harus tetap bisa menerima beberapa kontribusi dari donor yang sama dengan menambah `amount`.

## Spesifikasi Teknis Rust/Solana

Bagian ini menjadi rujukan teknis minimum untuk implementasi program Solana.

### Struktur Data Campaign

State utama campaign disimpan dalam account `Campaign`.

```rust
pub struct Campaign {
    pub creator: Pubkey,    // Who created this
    pub goal: u64,          // Target amount
    pub raised: u64,        // Current amount
    pub deadline: i64,      // When it ends
    pub claimed: bool,      // Already withdrawn?
}
```

Catatan implementasi:

- `creator` adalah wallet pembuat campaign dan satu-satunya signer yang boleh melakukan withdraw.
- `goal` dan `raised` menggunakan lamports.
- `deadline` memakai `i64` karena `Clock::unix_timestamp` di Solana juga bertipe `i64`.
- `claimed` mencegah withdraw ganda.
- Implementasi final kemungkinan perlu menambahkan `bump` atau metadata lain jika memakai Anchor PDA account.

### The Vault

Donasi tidak boleh dikirim langsung ke creator. Semua dana kontribusi harus masuk ke Program Derived Address (PDA) sebagai vault yang dikontrol program.

```rust
// Derive the vault address
let (vault_pda, bump) = Pubkey::find_program_address(
    &[b"vault", campaign_account.key.as_ref()],
    program_id
);

// Later, when transferring FROM the vault, use invoke_signed:
invoke_signed(
    &system_instruction::transfer(vault_pda, recipient, amount),
    &[vault_account, recipient_account, system_program],
    &[&[b"vault", campaign_account.key.as_ref(), &[bump]]]
)?;
```

Penjelasan:

- PDA adalah account yang alamatnya diturunkan secara deterministik dari seed dan `program_id`.
- PDA tidak memiliki private key.
- Program dapat "sign" untuk PDA menggunakan seed yang sama melalui `invoke_signed`.
- Dengan vault PDA, creator tidak dapat mengambil dana sebelum kondisi withdraw valid.
- Seed vault yang digunakan:

```text
["vault", campaign_account.key]
```

Aturan penggunaan vault:

- Saat `contribute`, transfer dilakukan dari donor ke vault PDA.
- Saat `withdraw`, transfer dilakukan dari vault PDA ke creator menggunakan `invoke_signed`.
- Saat `refund`, transfer dilakukan dari vault PDA ke donor menggunakan `invoke_signed`.

### Getting Current Time

Program harus menggunakan waktu on-chain dari Solana `Clock`, bukan timestamp dari client.

```rust
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

let clock = Clock::get()?;
let current_time = clock.unix_timestamp;
```

Penggunaan:

- `create_campaign`: valid jika `deadline > current_time`.
- `contribute`: valid jika `current_time < campaign.deadline`.
- `withdraw`: valid jika `current_time >= campaign.deadline`.
- `refund`: valid jika `current_time >= campaign.deadline`.

## Instruksi Program

### 1. Create Campaign

Creator membuat campaign baru.

Input:

```text
goal: u64
deadline: i64
```

Validasi:

- `deadline` harus lebih besar dari current unix timestamp.
- `goal` sebaiknya lebih besar dari `0`.

State yang disimpan:

```text
creator = creator.key()
goal = goal
deadline = deadline
raised = 0
claimed = false
```

Log:

```text
Campaign created: goal={goal}, deadline={deadline}
```

Catatan desain:

- Campaign PDA perlu seed yang stabil. Jika satu creator boleh membuat banyak campaign, gunakan campaign id atau counter.
- Untuk MVP paling sederhana, campaign dapat dibuat dengan seed:

```text
campaign = ["campaign", creator.key(), campaign_id]
```

`campaign_id` dapat berupa `u64` dari input atau timestamp yang diberikan client, tetapi lebih aman jika desain final memilih satu pendekatan eksplisit sebelum coding.

### 2. Contribute

Donor mengirim SOL ke campaign vault.

Input:

```text
amount: u64
```

Validasi:

- `amount` harus lebih besar dari `0`.
- Current time sebaiknya lebih kecil dari `deadline` agar campaign yang sudah berakhir tidak menerima kontribusi baru.
- Campaign belum `claimed`.

Logika:

- Transfer SOL dari donor ke campaign vault PDA.
- Update `campaign.raised += amount`.
- Buat atau update `Contribution Account`.
- Jika donor sudah pernah contribute, tambahkan `amount` ke contribution sebelumnya.

Log:

```text
Contributed: {amount} lamports, total={raised}
```

Catatan penting:

- Gunakan checked arithmetic untuk menghindari overflow pada `raised += amount`.
- Jangan transfer langsung ke creator.

### 3. Withdraw

Creator mengklaim dana jika campaign sukses.

Kondisi:

- `campaign.raised >= campaign.goal`
- Current time `>= campaign.deadline`
- Caller adalah `campaign.creator`
- Campaign belum pernah claimed

Logika:

- Transfer semua SOL yang tersedia di vault ke creator.
- Tandai `campaign.claimed = true`.

Log:

```text
Withdrawn: {amount} lamports
```

Catatan penting:

- Amount yang ditarik sebaiknya dihitung dari lamports vault yang tersedia, dengan tetap menjaga rent exemption jika vault berupa account yang harus tetap hidup.
- Jika vault adalah system account PDA tanpa data, desain close/transfer perlu dipastikan sesuai pola Anchor yang dipakai.
- Setelah `claimed = true`, refund tidak boleh dilakukan.

### 4. Refund

Donor mengambil kembali kontribusinya jika campaign gagal.

Kondisi:

- `campaign.raised < campaign.goal`
- Current time `>= campaign.deadline`
- Contribution milik donor ada.
- Contribution belum refunded.
- Campaign belum claimed.

Logika:

- Transfer jumlah contribution donor dari vault kembali ke donor.
- Tandai `contribution.refunded = true`.
- Set `contribution.amount = 0` setelah transfer agar state konsisten.

Log:

```text
Refunded: {amount} lamports
```

Catatan koreksi penting:

- Refund seharusnya mentransfer dana **dari vault ke donor**, bukan dari donor ke vault.
- Jika semua donor sudah refund, campaign account dapat dibiarkan tetap ada untuk audit trail atau ditutup pada fitur lanjutan.

## State Machine

```text
Draft/Created
  -> Active until deadline
  -> Ended Successful if raised >= goal
  -> Ended Failed if raised < goal

Ended Successful
  -> Withdrawn by creator

Ended Failed
  -> Refunded per donor
```

Aturan:

- Contribute hanya boleh sebelum deadline.
- Withdraw hanya boleh setelah deadline dan jika goal tercapai.
- Refund hanya boleh setelah deadline dan jika goal tidak tercapai.
- Campaign sukses tidak boleh refund.
- Campaign gagal tidak boleh withdraw.
- Campaign yang sudah claimed tidak boleh menerima kontribusi atau refund.

## Security And Correctness Notes

- Gunakan PDA untuk vault agar creator tidak dapat mengambil dana sebelum syarat terpenuhi.
- Gunakan `Clock` sysvar untuk membaca current unix timestamp.
- Gunakan checked arithmetic untuk semua penjumlahan lamports.
- Validasi semua signer dan ownership account.
- Pastikan contribution account selalu cocok dengan campaign dan donor.
- Cegah double refund dengan `refunded`.
- Cegah double withdraw dengan `claimed`.
- Jangan percaya timestamp dari client untuk validasi waktu.
- Jangan izinkan campaign menerima kontribusi setelah deadline.
- Pertimbangkan overflow dan underflow lamports.

## Error Cases Awal

Nama error yang disarankan:

```text
InvalidGoal
InvalidDeadline
CampaignEnded
CampaignNotEnded
CampaignNotSuccessful
CampaignNotFailed
UnauthorizedCreator
AlreadyClaimed
AlreadyRefunded
InvalidContributionAmount
ArithmeticOverflow
InsufficientVaultBalance
```

## Event / Log Strategy

Minimal sesuai requirement:

```text
Campaign created: goal={goal}, deadline={deadline}
Contributed: {amount} lamports, total={raised}
Withdrawn: {amount} lamports
Refunded: {amount} lamports
```

Jika menggunakan Anchor, event terstruktur juga bisa ditambahkan nanti:

```text
CampaignCreated
ContributionMade
CampaignWithdrawn
ContributionRefunded
```

Untuk MVP, log string requirement tetap harus dipertahankan agar behavior mudah diverifikasi.

## Testing Plan Awal

Unit/integration test yang harus ada saat implementasi:

1. Create campaign berhasil dengan deadline masa depan.
2. Create campaign gagal jika deadline sudah lewat.
3. Create campaign gagal jika goal `0`.
4. Contribute berhasil dan meningkatkan `raised`.
5. Contribute dari donor yang sama mengakumulasi contribution.
6. Contribute gagal jika amount `0`.
7. Contribute gagal setelah deadline.
8. Withdraw berhasil jika raised >= goal dan deadline lewat.
9. Withdraw gagal jika caller bukan creator.
10. Withdraw gagal jika goal belum tercapai.
11. Withdraw gagal sebelum deadline.
12. Withdraw gagal dua kali.
13. Refund berhasil jika goal gagal dan deadline lewat.
14. Refund gagal jika campaign sukses.
15. Refund gagal sebelum deadline.
16. Refund gagal dua kali.
17. Refund hanya mengembalikan amount milik donor terkait.

## QA Specification

Bagian ini digunakan sebagai checklist penerimaan kualitas untuk memastikan implementasi tidak menyimpang dari tujuan escrow crowdfunding.

### Success Criteria

- [ ] Accept campaign creation with goal and deadline.
- [ ] Accept contributions and track total raised.
- [ ] Allow withdrawal only if goal reached after deadline.
- [ ] Allow refunds only if goal not reached after deadline.
- [ ] Prevent double withdrawals.
- [ ] Use PDA for vault, not direct transfers to creator.

### Testing Checklist

Skenario happy path campaign sukses:

1. Create a campaign with `goal = 1000 SOL`, `deadline = tomorrow`.
2. Contribute `600 SOL`; should succeed and `raised = 600 SOL`.
3. Contribute `500 SOL`; should succeed and `raised = 1100 SOL`.
4. Try withdraw before deadline; should fail.
5. Wait until after deadline.
6. Withdraw should succeed.
7. Try withdraw again; should fail because campaign is already claimed.

Catatan:

- Nilai SOL pada checklist adalah skenario QA tingkat produk. Dalam implementasi dan test, nilai harus dikonversi ke lamports.
- Untuk automated test, "wait until after deadline" sebaiknya dibuat dengan deadline pendek atau manipulasi local validator/test context jika tersedia.

### Common Pitfalls

```text
Do not send donations directly to creator.
Do use PDA vault.

Do not allow withdrawal before deadline.
Do check both goal and time.

Do not forget to mark claimed = true.
Do prevent double withdrawals.

Do not use unwrap() everywhere.
Do handle errors properly.
```

## Resources

Referensi teknis yang harus digunakan saat implementasi:

1. **Program Derived Address (PDA)**

   URL:

   ```text
   https://solanacookbook.com/core-concepts/pdas.html
   ```

   Catatan:

   - Link Solana Cookbook ini saat ini mengarah ke dokumentasi Solana resmi tentang Program-Derived Address.
   - Gunakan referensi ini untuk memahami seed, bump, canonical bump, dan alasan PDA tidak memiliki private key.
   - Relevan langsung untuk desain vault:

   ```text
   ["vault", campaign_account.key]
   ```

2. **Cross Program Invocation (CPI)**

   URL:

   ```text
   https://solanacookbook.com/references/programs.html#how-to-do-cross-program-invocation
   ```

   Catatan:

   - Gunakan referensi ini untuk memahami cara program memanggil instruction program lain.
   - Relevan untuk transfer SOL melalui System Program.
   - Relevan untuk penggunaan `invoke` saat donor mengirim SOL ke vault.
   - Relevan untuk penggunaan `invoke_signed` saat program mentransfer SOL dari vault PDA ke creator atau donor.

3. **Clock / Current Time**

   Referensi terkait berada pada halaman Writing Programs Solana Cookbook:

   ```text
   https://solanacookbook.com/references/programs.html#how-to-get-clock-in-a-program
   ```

   Catatan:

   - Gunakan `Clock::get()?.unix_timestamp` sebagai sumber waktu on-chain.
   - Jangan menerima current time dari client untuk validasi deadline.

## Deliverables

Deliverables yang harus tersedia saat proyek dianggap selesai:

1. **Rust Program Code**

   Status awal: belum tersedia.

   Yang harus disediakan:

   - Source code program Solana dalam Rust.
   - Implementasi instruction:
     - `create_campaign`
     - `contribute`
     - `withdraw`
     - `refund`
   - Definisi account/state:
     - `Campaign`
     - `Contribution`
     - Vault PDA
   - Error handling tanpa penggunaan `unwrap()` yang tidak aman.
   - Test yang memverifikasi success criteria dan failure cases.

2. **Deployed To Solana Devnet**

   Status awal: belum tersedia.

   Yang harus disediakan:

   - Program berhasil di-build.
   - Program berhasil di-deploy ke Solana Devnet.
   - Network target yang digunakan:

   ```text
   devnet
   ```

   Bukti minimal:

   - Output deploy command.
   - Program address hasil deploy.
   - Explorer link Devnet jika tersedia.

3. **Program ID**

   Status awal: belum tersedia.

   Yang harus disediakan:

   ```text
   Program ID: <to be filled after deploy>
   ```

   Catatan:

   - Program ID harus dicatat setelah deploy berhasil.
   - Program ID harus konsisten dengan konfigurasi client/test.
   - Jika menggunakan Anchor, `Anchor.toml` dan `declare_id!()` harus selaras dengan Program ID hasil deploy.

4. **Test Transaction Signatures**

   Status awal: belum tersedia.

   Yang harus disediakan:

   - Signature transaksi create campaign.
   - Signature transaksi contribute.
   - Signature transaksi withdraw untuk campaign sukses.
   - Signature transaksi refund untuk campaign gagal.
   - Signature transaksi gagal tidak selalu tersedia sebagai finalized transaction, tetapi failure case tetap harus dibuktikan lewat test output.

   Format pencatatan:

   ```text
   Create Campaign Signature: <signature>
   Contribute Signature: <signature>
   Withdraw Signature: <signature>
   Refund Signature: <signature>
   ```

   Jika memungkinkan, tambahkan explorer link Devnet untuk setiap signature:

   ```text
   https://explorer.solana.com/tx/<signature>?cluster=devnet
   ```

## Open Decisions Sebelum Coding

Hal yang perlu diputuskan sebelum implementasi:

1. Nama final project: gunakan **VaultRaise** sementara.
2. Framework program: disarankan Anchor untuk ergonomi akun, PDA, dan test.
3. Seed campaign: perlu `campaign_id` eksplisit jika creator bisa membuat banyak campaign.
4. Apakah campaign account akan pernah ditutup atau dibiarkan sebagai audit trail.
5. Apakah platform akan mengambil fee di versi lanjutan.
6. Apakah campaign boleh menerima kontribusi setelah goal tercapai tetapi sebelum deadline.

Keputusan MVP yang disarankan:

- Gunakan Anchor.
- Gunakan `campaign_id: u64` saat create campaign.
- Izinkan kontribusi tetap masuk sebelum deadline walaupun goal sudah tercapai.
- Jangan tutup campaign account pada MVP.
- Jangan ada fee platform pada MVP.

## Acceptance Criteria MVP

Program dianggap sesuai konteks jika:

- Creator bisa membuat campaign dengan goal dan deadline.
- Donor bisa contribute SOL ke vault PDA, bukan ke creator.
- Total raised tercatat benar.
- Creator hanya bisa withdraw setelah deadline jika goal tercapai.
- Donor hanya bisa refund setelah deadline jika goal gagal.
- Dana tidak bisa diambil creator sebelum kondisi withdraw valid.
- Donor tidak bisa refund dua kali.
- Creator tidak bisa withdraw dua kali.
- Log sesuai requirement utama.
