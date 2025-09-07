@extends('layout.app')
@section('content')
    <div class="col overflow-auto px-3">
        <div class="row">
            <div class="col-xl-12 mt-3">
                <div class="card mx-3">
                    <div class="card-body">
                        <div class="row gy-4 px-3">
                            <div class="d-flex justify-content-between align-items-center mb-3">
                                <h5 class="mb-0">Load Information</h5>
                            </div>
                            <!-- Left Column -->
                            <div class="col-xl-12">
                                <div class="row g-3">
                                    <div class="col-sm-6">
                                        <label class="form-label">Title</label>
                                        <input class="form-control" id="title" type="text"
                                            value="{{ $load->title }}" readonly>
                                    </div>
                                    <div class="col-sm-6">
                                        <label class="form-label">Load Type</label>
                                        <input class="form-control" type="text" value="{{ $load->load_type->name }}"
                                            readonly>
                                    </div>
                                    <div class="col-sm-6">
                                        <label class="form-label">Equipment</label>
                                        <input class="form-control" type="text" value="{{ $load->equipment->name }}"
                                            readonly>
                                    </div>
                                    <div class="col-sm-6">
                                        <label class="form-label">Commodity Type</label>
                                        <input class="form-control" type="text" value="{{ $load->commodity_type->name }}"
                                            readonly>
                                    </div>
                                    <div class="col-sm-6">
                                        <label class="form-label">Weight</label>
                                        <input class="form-control" type="text" value="{{ $load->weight }}" readonly>
                                    </div>
                                    <div class="col-12">
                                        <label class="form-label">Special Instructions</label>
                                        <input class="form-control" type="text" value="{{ $load->special_instructions }}"
                                            readonly>
                                    </div>
                                </div>
                            </div>
                        </div> <!-- end row -->
                    </div>
                </div>
            </div>
            <div class="col-md-12">
                <form method="POST" action="{{ route('load.revise.save', $load->id) }}" enctype="multipart/form-data">
                    @csrf

                    <div class="card p-4 mx-4">
                        <div class="card-header py-2 d-flex justify-content-between align-items-center">
                            <h5 class="mb-2">Documents</h5>
                            <div>
                                <button type="button" id="add-doc-row" class="btn btn-primary btn-sm">Add Row</button>
                            </div>
                        </div>

                        <div class="card-body p-0">
                            <div class="table-responsive">
                                <div class="overflow-auto px-4" style="max-height: 260px;">
                                    <table class="table table-striped w-100 mb-0" id="docs-table">
                                        <thead class="sticky-top bg-white z-index-sticky">
                                            <tr>
                                                <th style="width:48px">#</th>
                                                <th>Document Name</th>
                                                <th style="width:160px">Type</th>
                                                <th style="width:280px">File</th>
                                                <th style="width:240px">Blockchain</th>
                                                <th style="width:110px">Action</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            @foreach ($load->loadDocuments as $i => $doc)
                                                <tr data-existing="1" data-hash="{{ $doc->hash ?? '' }}">
                                                    <td class="serial"></td>

                                                    {{-- Existing doc id (so backend can update) --}}
                                                    <input type="hidden" name="doc_id[]" value="{{ $doc->id }}" />

                                                    <td>
                                                        <input type="text" name="doc_name[]" class="form-control"
                                                            value="{{ $doc->document_name }}" required>
                                                    </td>

                                                    <td>
                                                        <select name="doc_type[]" class="form-select doc-type" required>
                                                            <option value="standard"
                                                                {{ $doc->document_type === 'standard' ? 'selected' : '' }}>
                                                                Standard</option>
                                                            <option value="blockchain"
                                                                {{ $doc->document_type === 'blockchain' ? 'selected' : '' }}>
                                                                Blockchain</option>
                                                        </select>
                                                    </td>

                                                    <td>
                                                        <div class="d-grid gap-1">
                                                            <input type="file" name="documents[]"
                                                                class="form-control doc-file"
                                                                accept=".pdf,.jpg,.jpeg,.png,.docx,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,image/jpeg,image/png">
                                                            <small class="text-muted">
                                                                Current:
                                                                {{ $doc->original_name ?? basename($doc->file_path) }}
                                                            </small>
                                                        </div>
                                                    </td>

                                                    <td class="bc-cell">
                                                        @if ($doc->document_type === 'blockchain')
                                                            <div class="d-flex align-items-center gap-2">
                                                                <small class="text-muted">Stored
                                                                    hash:</small>
                                                                <code
                                                                    class="small">{{ $doc->hash ? substr($doc->hash, 0, 12) . '…' : '—' }}</code>
                                                                <button type="button"
                                                                    class="btn btn-outline-secondary btn-sm verify-btn">Verify</button>
                                                            </div>
                                                        @else
                                                            <span class="text-muted">—</span>
                                                        @endif
                                                    </td>

                                                    <td>
                                                        <button type="button"
                                                            class="btn btn-danger btn-sm remove-row">Remove</button>
                                                    </td>
                                                </tr>
                                            @endforeach
                                        </tbody>
                                    </table>
                                </div>
                                @if ($load->latestLegs->status_id == 7 || $load->latestLegs->status_id == 1)
                                    <div class="p-3 text-end">
                                        <button type="submit" class="btn btn-primary">Save
                                            Changes</button>
                                    </div>
                                @endif
                            </div>
                        </div>
                    </div>
                </form>

                {{-- Simple verify modal --}}
                <div class="modal fade" id="verifyModal" tabindex="-1" aria-hidden="true">
                    <div class="modal-dialog">
                        <div class="modal-content">
                            <div class="modal-header">
                                <h6 class="modal-title">Verify Blockchain Document</h6>
                                <button type="button" class="btn-close" data-bs-dismiss="modal"
                                    aria-label="Close"></button>
                            </div>
                            <div class="modal-body">
                                <input type="file" id="verifyFile" class="form-control mb-2"
                                    accept=".pdf,.jpg,.jpeg,.png,.docx,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,image/jpeg,image/png">
                                <div class="small text-muted">We compute a local SHA-256 and
                                    compare with the stored hash.</div>
                                <div id="verifyResult" class="mt-3"></div>
                            </div>
                            <div class="modal-footer">
                                <button class="btn btn-secondary" data-bs-dismiss="modal">Close</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <script src="{{ url('assets/js/jquery.min.js') }}"></script>

    <script>
        (function() {
            const table = document.getElementById('docs-table').querySelector('tbody');
            const addBtn = document.getElementById('add-doc-row');
            const ACCEPT =
                '.pdf,.jpg,.jpeg,.png,.docx,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,image/jpeg,image/png';

            function renumber() {
                table.querySelectorAll('tr .serial').forEach((td, i) => td.textContent = i + 1);
                // show blockchain cell if type==blockchain
                table.querySelectorAll('tr').forEach(tr => {
                    const sel = tr.querySelector('.doc-type');
                    const bcCell = tr.querySelector('.bc-cell');
                    if (!sel || !bcCell) return;
                    if (sel.value === 'blockchain') {
                        if (!bcCell.querySelector('.verify-btn')) {
                            bcCell.innerHTML = `
                        <div class="d-flex align-items-center gap-2">
                            <small class="text-muted">Stored hash:</small>
                            <code class="small">${(tr.dataset.hash || '—').substring(0,12)}${tr.dataset.hash ? '…' : ''}</code>
                            <button type="button" class="btn btn-outline-secondary btn-sm verify-btn">Verify</button>
                        </div>`;
                        }
                    } else {
                        bcCell.innerHTML = '<span class="text-muted">—</span>';
                    }
                });
            }

            function addRow() {
                const tr = document.createElement('tr');
                tr.innerHTML = `
                    <td class="serial"></td>
                    <input type="hidden" name="doc_id[]" value="">
                    <td><input type="text" name="doc_name[]" class="form-control" placeholder="Document name" required></td>
                    <td>
                        <select name="doc_type[]" class="form-select doc-type" required>
                            <option value="standard" selected>Standard</option>
                            <option value="blockchain">Blockchain</option>
                        </select>
                    </td>
                    <td>
                        <div class="d-grid gap-1">
                            <input type="file" name="documents[]" class="form-control doc-file" accept="${ACCEPT}" required>
                            <small class="text-muted">Choose a file</small>
                        </div>
                    </td>
                    <td class="bc-cell"><span class="text-muted">—</span></td>
                    <td><button type="button" class="btn btn-danger btn-sm remove-row">Remove</button></td>
                `;
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
                    if (rows.length <= 1) {
                        // keep at least one row
                        return;
                    }
                    e.target.closest('tr').remove();
                    renumber();
                } else if (e.target.classList.contains('verify-btn')) {
                    const tr = e.target.closest('tr');
                    openVerifyModal(tr.dataset.hash || '');
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
                    `<div class="small">Stored hash: <code>${storedHash}</code></div>` :
                    `<div class="small text-warning">No stored hash on this document yet.</div>`;
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
                    const hex = Array.from(new Uint8Array(digest)).map(b => b.toString(16).padStart(2, '0'))
                        .join('');
                    const match = storedHash && hex === storedHash;
                    verifyResult.innerHTML = `
                <div class="small">Computed: <code>${hex}</code></div>
                ${storedHash
                    ? (match
                        ? '<div class="mt-2 alert alert-success py-1 mb-0">Verified ✓</div>'
                        : '<div class="mt-2 alert alert-danger  py-1 mb-0">Mismatch ✗</div>')
                    : '<div class="mt-2 alert alert-info py-1 mb-0">Hash computed. This will be saved after you upload as blockchain.</div>'
                }`;
                } catch (err) {
                    verifyResult.innerHTML =
                        `<div class="text-danger">Failed to compute hash: ${err}</div>`;
                }
            });
        })();
    </script>
@endsection
