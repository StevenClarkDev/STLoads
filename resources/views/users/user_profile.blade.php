@extends('layout.app')
@section('content')
    <div class="col overflow-auto px-3">
        <div class="row">
            <div class="col-xl-12 mt-3">
                <div class="card mx-3">
                    <div class="card-body">
                        <div class="row gy-4 px-3">
                            <div class="d-flex justify-content-between align-items-center mb-3">
                                <h5 class="mb-0">User Information</h5>
                            </div>
                            <!-- Left Column -->
                            <div class="col-xl-8">
                                <div class="row g-3">
                                    <div class="col-sm-6">
                                        <label class="form-label">Name</label>
                                        <input class="form-control" id="userName" type="text"
                                            value="{{ $user->name }}" readonly>
                                    </div>
                                    <div class="col-sm-6">
                                        <label class="form-label">Role</label>
                                        <input class="form-control" type="text"
                                            value="{{ $user->getRoleNames()->implode(', ') }}" readonly>
                                    </div>
                                    <div class="col-sm-6">
                                        <label class="form-label">Email</label>
                                        <input class="form-control" type="email" value="{{ $user->email }}" readonly>
                                    </div>
                                    <div class="col-sm-6">
                                        <label class="form-label">DOB</label>
                                        <input class="form-control" type="date" value="{{ $user->dob }}" readonly>
                                    </div>
                                    <div class="col-sm-6">
                                        <label class="form-label">Gender</label>
                                        <input class="form-control" type="text" value="{{ $user->gender }}" readonly>
                                    </div>
                                    <div class="col-sm-6">
                                        <label class="form-label">DOJ</label>
                                        <input class="form-control" type="text" value="{{ $user->created_at }}" readonly>
                                    </div>
                                    <div class="col-12">
                                        <label class="form-label">Address</label>
                                        <input class="form-control" type="text" value="{{ $user->address }}" readonly>
                                    </div>
                                </div>
                            </div>

                            <!-- Right Column -->
                            <div class="col-xl-4">
                                <div class="card social-profile mb-0">
                                    <div class="card-body bg-primary rounded-4">
                                        <div class="social-img-wrap my-2">
                                            <div class="social-img">
                                                <img class="img-fluid rounded-circle"
                                                    src="{{ $user->image ? route('admin.serve-kyc-file', ['path' => $user->image]) : asset('assets/images/default-avatar.png') }}" alt="profile">
                                            </div>
                                            <div class="edit-icon">
                                                <svg>
                                                    <use href="../assets/svg/icon-sprite.svg#profile-check">
                                                    </use>
                                                </svg>
                                            </div>
                                        </div>
                                        <div class="social-details text-white text-center">
                                            <h5 class="mb-1 text-white">{{ $user->name }}</h5>
                                            <span class="text-light mb-4 d-block">{{ $user->email }}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div> <!-- end row -->
                    </div>
                </div>
            </div>
            <div class="col-md-12">
                                  <form method="POST" action="{{ route('profile.revise.save', $user->id) }}"
                                      enctype="multipart/form-data">
                                      @csrf

                                      <div class="card p-4 mx-4">
                                          <div
                                              class="card-header py-2 d-flex justify-content-between align-items-center">
                                              <h5 class="mb-2">Documents</h5>
                                              <div>
                                                  <button type="button" id="add-doc-row"
                                                      class="btn btn-primary btn-sm">Add Row</button>
                                              </div>
                                          </div>

                                          <div class="card-body p-0">
                                            <div class="table-responsive">
                                              <div class="overflow-auto px-4">
                                                <table class="table table-striped w-100 mb-0" id="docs-table">
                                                  <thead class="sticky-top bg-white z-index-sticky">
                                                    <tr>
                                                      <th style="width:48px">#</th>
                                                      <th style="width:160px">Document Name</th>
                                                      <th style="width:160px">Type</th>
                                                      <th style="width:220px">File</th>
                                                      <th style="width:280px">Blockchain</th>
                                                      <th style="width:110px">Action</th>
                                                    </tr>
                                                  </thead>
                                                  <tbody>
                                                    @foreach ($user->kycDocuments as $doc)
                                                      <tr data-existing="1" data-hash="{{ $doc->hash ?? '' }}">
                                                        <td class="serial"></td>
                                                        <input type="hidden" name="doc_id[]" value="{{ $doc->id }}" />

                                                        <td><input type="text" name="doc_name[]" class="form-control form-control-sm"
                                                            value="{{ $doc->document_name }}" required></td>

                                                        <td>
                                                          <select name="doc_type[]" class="form-select form-select-sm doc-type" required>
                                                            <option value="standard"
                                                              {{ $doc->document_type === 'standard' ? 'selected' : '' }}>Standard</option>
                                                            <option value="blockchain"
                                                              {{ $doc->document_type === 'blockchain' ? 'selected' : '' }}>Blockchain</option>
                                                          </select>
                                                        </td>

                                                        <td>
                                                          <div class="input-group input-group-sm">
                                                            <input type="text" class="form-control"
                                                              value="{{ $doc->original_name ?? basename($doc->file_path) }}" readonly>
                                                            <label class="btn btn-outline-primary btn-sm mb-0 d-flex align-items-center justify-content-center"
                                                              style="width: 36px;">
                                                              <i class="bi bi-arrow-repeat"></i>
                                                              <input type="file" name="documents[]" class="d-none doc-file"
                                                                accept=".pdf,.jpg,.jpeg,.png,.docx,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,image/jpeg,image/png">
                                                            </label>
                                                          </div>
                                                        </td>

                                                        <td class="bc-cell">
                                                          @if ($doc->document_type === 'blockchain')
                                                          <div class="input-group input-group-sm">
                                                            <input type="text" class="form-control text-monospace"
                                                              value="{{ $doc->hash ? substr($doc->hash, 0, 35) . '…' : '—' }}" readonly>
                                                            <button type="button"
                                                              class="btn btn-outline-primary btn-sm d-flex align-items-center justify-content-center verify-btn"
                                                              style="width: 36px;" data-bs-toggle="modal" data-bs-target="#verifyModal"
                                                              data-hash="{{ $doc->hash ?? '' }}">
                                                              <i class="bi bi-shield-check"></i>
                                                            </button>
                                                          </div>
                                                          @else
                                                          <span class="text-muted">—</span>
                                                          @endif
                                                        </td>

                                                        <td><button type="button" class="btn btn-danger btn-sm remove-row">Remove</button></td>
                                                      </tr>
                                                    @endforeach
                                                  </tbody>
                                                </table>
                                              </div>

                                              <div class="p-3 text-end">
                                                <button type="submit" class="btn btn-primary">Save Changes</button>
                                              </div>
                                            </div>
                                          </div>
                                      </div>
                                  </form>
                                  <!-- Verify Modal -->
                                  <div class="modal fade" id="verifyModal" tabindex="-1" aria-hidden="true">
                                      <div class="modal-dialog modal-dialog-centered modal-lg">
                                          <div class="modal-content border-0 shadow-lg rounded-4 p-2">
                                              <div class="modal-header border-0 pb-0">
                                                  <h5 class="modal-title">Verify Blockchain Document</h5>
                                                  <button type="button" class="btn-close" data-bs-dismiss="modal"
                                                  aria-label="Close"></button>
                                              </div>
                                              <div class="modal-body">
                                                  <div class="mb-3">
                                                  <label class="form-label">Upload Document</label>
                                                  <input type="file" id="verifyFile" class="form-control"
                                                      accept=".pdf,.jpg,.jpeg,.png,.docx,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,image/jpeg,image/png">
                                                  <div class="form-text text-muted">We compute a local SHA-256 and compare with the stored hash.</div>
                                                  </div>

                                                  <!-- Stored hash / verification result -->
                                                  <div id="verifyResult" class="mt-3"></div>
                                              </div>
                                          </div>
                                      </div>
                                  </div>

                              </div>
        </div>
    </div>
    <!-- Theme js-->
      <script src="{{ url('assets/js/script.js') }}"></script>
      <script src="{{ url('assets/js/theme-customizer/customizer.js') }}"></script>
      <script>
          new WOW().init();
      </script>
<script src="{{ url('assets/js/jquery.min.js') }}"></script>

      <script>
(function() {
    const table = document.getElementById('docs-table').querySelector('tbody');
    const addBtn = document.getElementById('add-doc-row');
    const ACCEPT =
        '.pdf,.jpg,.jpeg,.png,.docx,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,image/jpeg,image/png';

    function renumber() {
        table.querySelectorAll('tr .serial').forEach((td, i) => td.textContent = i + 1);

        table.querySelectorAll('tr').forEach(tr => {
            const sel = tr.querySelector('.doc-type');
            const bcCell = tr.querySelector('.bc-cell');
            if (!sel || !bcCell) return;

            if (sel.value === 'blockchain') {
                // keep Blade's blockchain UI intact
                if (bcCell.querySelector('.verify-btn')) {
                    bcCell.querySelector('.verify-btn').classList.remove('d-none');
                }
            } else {
                // replace with simple dash if not blockchain
                bcCell.innerHTML = '<span class="text-muted">—</span>';
            }
        });
    }

    function addRow() {
        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td class="serial"></td>
          <input type="hidden" name="doc_id[]" value="">
          <td><input type="text" name="doc_name[]" class="form-control form-control-sm" placeholder="Document name" required></td>
          <td>
            <select name="doc_type[]" class="form-select form-select-sm doc-type" required>
              <option value="standard" selected>Standard</option>
              <option value="blockchain">Blockchain</option>
            </select>
          </td>
          <td>
            <div class="input-group input-group-sm">
              <input type="text" class="form-control" placeholder="No file chosen" readonly>
              <label class="btn btn-outline-primary btn-sm mb-0 d-flex align-items-center justify-content-center" style="width: 36px;">
                <i class="bi bi-upload"></i>
                <input type="file" name="documents[]" class="d-none doc-file" accept="${ACCEPT}" required>
              </label>
            </div>
          </td>
          <td class="bc-cell"><span class="text-muted">—</span></td>
          <td><button type="button" class="btn btn-danger btn-sm remove-row">Remove</button></td>`;
        table.appendChild(tr);
        renumber();
    }

    table.addEventListener('change', function(e) {
        if (e.target.classList.contains('doc-type')) {
            renumber();
        }
    });

    table.addEventListener('click', function(e) {
    if (e.target.classList.contains('remove-row')) {
        const rows = table.querySelectorAll('tr');
        if (rows.length <= 1) return;
        e.target.closest('tr').remove();
        renumber();
    } else if (e.target.closest('.verify-btn')) {
        const btn = e.target.closest('.verify-btn');
        openVerifyModal(btn.dataset.hash || '');
    }
});


    addBtn.addEventListener('click', addRow);
    renumber();

    // ---- Blockchain verify (client-side SHA-256 with Web Crypto) ----
    const verifyModalEl = document.getElementById('verifyModal');
    const verifyFile = document.getElementById('verifyFile');
    const verifyResult = document.getElementById('verifyResult');
    let verifyModal, storedHash = '';

    function openVerifyModal(hash) {
        storedHash = (hash || '').toLowerCase();
        verifyResult.innerHTML = storedHash ?
            `<div class="mt-4">Stored hash: <span class="badge bg-primary rounded-pill py-2">${storedHash}</span></div>` :
            `<div class="mt-4 text-warning">No stored hash on this document yet.</div>`;
        verifyFile.value = '';
        verifyModal = new bootstrap.Modal(verifyModalEl);
        verifyModal.show();
    }

    verifyFile.addEventListener('change', async function() {
        verifyResult.innerHTML = '<div class="text-muted">Computing hash…</div>';
        const file = this.files[0];
        if (!file) return;
        try {
            const ab = await file.arrayBuffer();
            const digest = await crypto.subtle.digest('SHA-256', ab);
            const hex = Array.from(new Uint8Array(digest)).map(b => b.toString(16).padStart(2, '0')).join('');
            const match = storedHash && hex === storedHash;
            verifyResult.innerHTML = `
                <div class="small">Computed: <code>${hex}</code></div>
                ${storedHash
                    ? (match
                        ? '<div class="my-2 alert alert-success py-1 mb-0"><i class="fa fa-check-circle me-1"></i> Verified</div>'
                        : '<div class="mt-2 alert alert-danger py-1 mb-0"><i class="fa fa-exclamation-triangle me-1"></i> Mismatch</div>')
                    : '<div class="mt-2 alert alert-info py-1 mb-0">Hash computed. This will be saved after you upload as blockchain.</div>'
                }`;
        } catch (err) {
            verifyResult.innerHTML =
                `<div class="text-danger">Failed to compute hash: ${err}</div>`;
        }
    });

    verifyModalEl.addEventListener('hidden.bs.modal', () => {
        document.querySelectorAll('.modal-backdrop').forEach(el => el.remove());
        document.body.classList.remove('modal-open');
        document.body.style.removeProperty('padding-right');
    });

})();
</script>

@endsection
