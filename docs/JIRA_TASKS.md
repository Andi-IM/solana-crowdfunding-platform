# VaultRaise - Jira Task Breakdown

Dokumen ini berisi daftar task bergaya Jira untuk mengerjakan MVP Solana Crowdfunding Platform. Nama kerja proyek: **VaultRaise**.

## EPIC-001 - Project Foundation

### VR-001 - Initialize Solana/Anchor Project Structure

**Type:** Task  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Set up struktur awal proyek Solana menggunakan Anchor agar program, test, dan deployment dapat dikelola secara konsisten.

**Acceptance Criteria:**

- Anchor project berhasil dibuat.
- Struktur folder program Rust tersedia.
- `Anchor.toml` tersedia.
- Project bisa menjalankan build awal.
- File AI lokal tetap tidak ter-track oleh git.

**Dependencies:** None

### VR-002 - Define Program Accounts And Error Types

**Type:** Task  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Implementasikan struktur akun utama untuk campaign, contribution, dan error custom.

**Acceptance Criteria:**

- Account `Campaign` memiliki `creator`, `goal`, `raised`, `deadline`, dan `claimed`.
- Account `Contribution` menyimpan campaign, donor, amount, dan refunded status.
- Error custom tersedia untuk invalid deadline, unauthorized creator, already claimed, already refunded, arithmetic overflow, dan invalid amount.
- Tidak ada penggunaan `unwrap()` untuk flow yang bisa gagal.

**Dependencies:** VR-001

## EPIC-002 - Campaign Lifecycle

### VR-003 - Implement Create Campaign Instruction

**Type:** Story  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Sebagai creator, saya ingin membuat campaign dengan goal dan deadline agar donor dapat berkontribusi ke campaign tersebut.

**Acceptance Criteria:**

- Instruction menerima `goal: u64` dan `deadline: i64`.
- Program menolak `goal = 0`.
- Program menolak deadline yang tidak berada di masa depan.
- Campaign menyimpan creator, goal, deadline, raised `0`, dan claimed `false`.
- Program menulis log: `Campaign created: goal={goal}, deadline={deadline}`.

**Dependencies:** VR-002

### VR-004 - Implement PDA Vault Derivation

**Type:** Task  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Implementasikan vault PDA agar semua dana campaign dikunci oleh program, bukan dikirim langsung ke creator.

**Acceptance Criteria:**

- Vault PDA diturunkan dengan seed `["vault", campaign.key()]`.
- Bump disimpan atau dapat diverifikasi secara aman.
- Program memvalidasi vault account yang diberikan sesuai seed.
- Dokumentasi internal menjelaskan bahwa creator tidak boleh menerima donasi langsung.

**Dependencies:** VR-003

### VR-005 - Implement Contribute Instruction

**Type:** Story  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 2 days  

**Description:**  
Sebagai donor, saya ingin mengirim SOL ke campaign vault agar dana saya terkunci sampai campaign sukses atau gagal.

**Acceptance Criteria:**

- Instruction menerima `amount: u64`.
- Program menolak `amount = 0`.
- Program menolak kontribusi setelah deadline.
- Program mentransfer SOL dari donor ke vault PDA.
- Program menambah `campaign.raised` menggunakan checked arithmetic.
- Program membuat atau memperbarui account `Contribution`.
- Program menulis log: `Contributed: {amount} lamports, total={raised}`.

**Dependencies:** VR-004

### VR-006 - Implement Withdraw Instruction

**Type:** Story  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 2 days  

**Description:**  
Sebagai creator, saya ingin menarik dana dari vault jika campaign mencapai goal setelah deadline.

**Acceptance Criteria:**

- Withdraw hanya berhasil jika `raised >= goal`.
- Withdraw hanya berhasil jika current time `>= deadline`.
- Withdraw hanya berhasil jika caller adalah creator.
- Withdraw gagal jika campaign sudah claimed.
- Program mentransfer SOL dari vault PDA ke creator menggunakan signed PDA flow.
- Program menandai `claimed = true`.
- Program menulis log: `Withdrawn: {amount} lamports`.

**Dependencies:** VR-005

### VR-007 - Implement Refund Instruction

**Type:** Story  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 2 days  

**Description:**  
Sebagai donor, saya ingin mendapatkan refund jika campaign gagal mencapai goal setelah deadline.

**Acceptance Criteria:**

- Refund hanya berhasil jika `raised < goal`.
- Refund hanya berhasil jika current time `>= deadline`.
- Refund hanya berhasil untuk donor yang memiliki contribution.
- Refund gagal jika contribution sudah refunded.
- Program mentransfer SOL dari vault PDA ke donor menggunakan signed PDA flow.
- Program menandai contribution sebagai refunded.
- Program menulis log: `Refunded: {amount} lamports`.

**Dependencies:** VR-005

## EPIC-003 - Testing And QA

### VR-008 - Write Unit And Integration Tests For Campaign Creation

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Buat test untuk memastikan campaign creation valid dan invalid berjalan sesuai spesifikasi.

**Acceptance Criteria:**

- Test create campaign berhasil dengan deadline masa depan.
- Test create campaign gagal jika deadline sudah lewat.
- Test create campaign gagal jika goal `0`.
- State awal campaign tervalidasi.

**Dependencies:** VR-003

### VR-009 - Write Tests For Contribution Flow

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Buat test kontribusi donor dan akumulasi total raised.

**Acceptance Criteria:**

- Contribute `600 SOL` equivalent in lamports berhasil.
- Contribute tambahan `500 SOL` equivalent in lamports berhasil.
- `raised` menjadi total `1100 SOL` equivalent in lamports.
- Contribution donor tercatat benar.
- Contribute setelah deadline gagal.
- Contribute amount `0` gagal.

**Dependencies:** VR-005

### VR-010 - Write Tests For Withdraw Flow

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Buat test withdraw untuk campaign sukses dan failure cases.

**Acceptance Criteria:**

- Withdraw sebelum deadline gagal.
- Withdraw setelah deadline dan goal tercapai berhasil.
- Withdraw oleh non-creator gagal.
- Withdraw kedua gagal karena already claimed.
- Balance creator bertambah sesuai dana vault.

**Dependencies:** VR-006

### VR-011 - Write Tests For Refund Flow

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Buat test refund untuk campaign gagal dan failure cases.

**Acceptance Criteria:**

- Refund sebelum deadline gagal.
- Refund setelah deadline dan goal tidak tercapai berhasil.
- Refund untuk campaign sukses gagal.
- Refund kedua gagal karena already refunded.
- Refund hanya mengembalikan amount milik donor terkait.

**Dependencies:** VR-007

### VR-012 - Run QA Checklist And Capture Evidence

**Type:** Task  
**Priority:** High  
**Assignee:** QA Engineer  
**Estimate:** 1 day  

**Description:**  
Jalankan checklist QA yang ada di project context dan catat hasilnya sebagai bukti sebelum deploy.

**Acceptance Criteria:**

- Semua success criteria QA ditandai pass/fail.
- Skenario campaign sukses dijalankan end-to-end.
- Skenario campaign gagal refund dijalankan end-to-end.
- Semua failure case penting memiliki test evidence.
- Tidak ada transfer langsung ke creator saat contribute.

**Dependencies:** VR-008, VR-009, VR-010, VR-011

## EPIC-004 - Devnet Deployment

### VR-013 - Prepare Devnet Wallet And Configuration

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 0.5 day  

**Description:**  
Siapkan konfigurasi wallet dan cluster Devnet untuk deployment program.

**Acceptance Criteria:**

- Solana CLI target cluster adalah Devnet.
- Wallet deployer tersedia.
- Wallet memiliki SOL Devnet yang cukup.
- Konfigurasi Anchor mengarah ke Devnet.

**Dependencies:** VR-012

### VR-014 - Deploy Program To Solana Devnet

**Type:** Task  
**Priority:** Highest  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Deploy program Rust ke Solana Devnet dan catat Program ID.

**Acceptance Criteria:**

- Program berhasil di-build untuk deploy.
- Program berhasil di-deploy ke Devnet.
- Program ID dicatat di dokumentasi deliverables.
- Jika menggunakan Anchor, `declare_id!()` dan `Anchor.toml` selaras dengan Program ID.
- Explorer link Devnet tersedia jika memungkinkan.

**Dependencies:** VR-013

### VR-015 - Execute Devnet Test Transactions

**Type:** Task  
**Priority:** High  
**Assignee:** Blockchain Engineer  
**Estimate:** 1 day  

**Description:**  
Jalankan transaksi Devnet untuk membuktikan campaign creation, contribution, withdraw, dan refund.

**Acceptance Criteria:**

- Signature transaksi create campaign dicatat.
- Signature transaksi contribute dicatat.
- Signature transaksi withdraw campaign sukses dicatat.
- Signature transaksi refund campaign gagal dicatat.
- Explorer link Devnet untuk setiap signature dicatat jika tersedia.

**Dependencies:** VR-014

## EPIC-005 - Documentation And Handoff

### VR-016 - Update Project Context With Implementation Decisions

**Type:** Task  
**Priority:** Medium  
**Assignee:** Technical Writer / Blockchain Engineer  
**Estimate:** 0.5 day  

**Description:**  
Update dokumen konteks dengan keputusan final yang muncul selama implementasi.

**Acceptance Criteria:**

- Nama project final atau nama kerja dikonfirmasi.
- Campaign seed final terdokumentasi.
- Program ID terdokumentasi.
- Devnet deployment evidence terdokumentasi.
- Test transaction signatures terdokumentasi.

**Dependencies:** VR-015

### VR-017 - Prepare Developer Handoff Notes

**Type:** Task  
**Priority:** Medium  
**Assignee:** Technical Writer / Project Manager  
**Estimate:** 0.5 day  

**Description:**  
Siapkan catatan handoff agar programmer berikutnya dapat menjalankan build, test, dan deploy tanpa kehilangan konteks.

**Acceptance Criteria:**

- Instruksi setup local environment tersedia.
- Instruksi build tersedia.
- Instruksi test tersedia.
- Instruksi deploy Devnet tersedia.
- Known limitations MVP terdokumentasi.

**Dependencies:** VR-016

## Delivery Milestones

1. **Milestone 1 - Foundation Ready**
   - VR-001
   - VR-002

2. **Milestone 2 - Core Program Complete**
   - VR-003
   - VR-004
   - VR-005
   - VR-006
   - VR-007

3. **Milestone 3 - Test Coverage Complete**
   - VR-008
   - VR-009
   - VR-010
   - VR-011
   - VR-012

4. **Milestone 4 - Devnet Delivered**
   - VR-013
   - VR-014
   - VR-015

5. **Milestone 5 - Handoff Complete**
   - VR-016
   - VR-017

## Definition Of Done

Proyek MVP dianggap selesai jika:

- Rust program code tersedia.
- Semua instruction utama selesai: create campaign, contribute, withdraw, refund.
- Test utama dan failure cases lulus.
- Program berhasil deploy ke Solana Devnet.
- Program ID tercatat.
- Test transaction signatures tercatat.
- Tidak ada dana kontribusi yang dikirim langsung ke creator.
- Vault PDA digunakan untuk escrow dana.
- Dokumentasi konteks dan handoff diperbarui.

