@extends('admin-layout.app')
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
                <div class="card p-4 mx-4">
                    <div class="card-header py-2 d-flex justify-content-between align-items-center">
                        <h5 class="mb-2">Documents</h5>
                    </div>

                    <div class="card-body p-0">
                        <div class="table-responsive">
                            <!-- Use Bootstrap class-based height and overflow utilities -->
                            <div class="overflow-auto px-4" style="max-height: 200px;">
                                @php
                                    use Illuminate\Support\Facades\Storage;

                                    $imageMimes = ['image/jpeg', 'image/png'];
                                    $pdfMimes = ['application/pdf'];
                                    $docxMimes = [
                                        'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
                                    ];

                                    function human_filesize($bytes, $decimals = 1)
                                    {
                                        if ($bytes === null) {
                                            return '—';
                                        }
                                        $size = ['B', 'KB', 'MB', 'GB', 'TB'];
                                        $factor = $bytes > 0 ? floor((strlen((string) $bytes) - 1) / 3) : 0;
                                        return sprintf("%.{$decimals}f", $bytes / pow(1024, $factor)) .
                                            ' ' .
                                            $size[$factor];
                                    }
                                @endphp

                                <table class="table table-striped w-100 mb-0" id="user-approval-table">
                                    <thead class="sticky-top bg-white z-index-sticky">
                                        <tr>
                                            <th style="width:60px">#</th>
                                            <th>Document Name</th>
                                            <th style="width:120px">Type</th>
                                            <th style="width:220px">Preview</th>
                                            <th style="width:110px">Size</th>
                                            <th style="width:130px">Action</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        @forelse ($load->loadDocuments as $i => $doc)
                                            @php
                                                $exists =
                                                    $doc->file_path && Storage::disk('public')->exists($doc->file_path);
                                                $url = $exists ? $doc->file_url : null;
                                                $ext = $doc->original_name
                                                    ? strtolower(pathinfo($doc->original_name, PATHINFO_EXTENSION))
                                                    : null;
                                                $mime = $doc->mime_type;
                                            @endphp
                                            <tr>
                                                <td>{{ $loop->iteration }}</td>

                                                <td>
                                                    <div class="fw-semibold">
                                                        {{ $doc->document_name ?? 'Untitled' }}
                                                        @if (($doc->document_type ?? '') === 'blockchain')
                                                            <span class="badge bg-dark ms-2">Blockchain</span>
                                                        @endif
                                                    </div>
                                                    <div class="text-muted small">
                                                        {{ $doc->original_name ?? basename($doc->file_path) }}
                                                        @if (($doc->document_type ?? '') === 'blockchain' && $doc->hash)
                                                            <span class="ms-2">• hash:
                                                                <code>{{ Str::limit($doc->hash, 12, '…') }}</code></span>
                                                        @endif
                                                    </div>
                                                </td>

                                                <td class="text-nowrap">
                                                    @if ($ext)
                                                        {{ strtoupper($ext) }}
                                                    @elseif($mime)
                                                        {{ $mime }}
                                                    @else
                                                        —
                                                    @endif
                                                </td>

                                                <td>
                                                    @if (!$exists)
                                                        <span class="badge bg-danger">Missing file</span>
                                                    @else
                                                        @if (in_array($mime, $imageMimes))
                                                            <a href="{{ $url }}" target="_blank" rel="noopener">
                                                                <img src="{{ $url }}" alt="preview"
                                                                    class="img-thumbnail"
                                                                    style="max-width: 110px; max-height: 110px;">
                                                            </a>
                                                        @elseif(in_array($mime, $pdfMimes))
                                                            <a href="{{ $url }}" target="_blank" rel="noopener"
                                                                class="btn btn-outline-secondary btn-sm">
                                                                View PDF
                                                            </a>
                                                        @elseif(in_array($mime, $docxMimes) || $ext === 'docx')
                                                            <span class="text-muted me-2">DOCX (no preview)</span>
                                                            <a href="{{ $url }}" target="_blank" rel="noopener"
                                                                class="btn btn-outline-secondary btn-sm">
                                                                Open
                                                            </a>
                                                        @else
                                                            <a href="{{ $url }}" target="_blank" rel="noopener"
                                                                class="btn btn-outline-secondary btn-sm">
                                                                Open
                                                            </a>
                                                        @endif
                                                    @endif
                                                </td>

                                                <td>{{ human_filesize($doc->file_size) }}</td>

                                                <td>
                                                    @if ($exists)
                                                        <a href="{{ $url }}"
                                                            download="{{ $doc->original_name ?? basename($doc->file_path) }}"
                                                            class="btn btn-primary btn-sm">
                                                            Download
                                                        </a>
                                                    @else
                                                        <button class="btn btn-secondary btn-sm" disabled>Download</button>
                                                    @endif
                                                </td>
                                            </tr>
                                        @empty
                                            <tr>
                                                <td colspan="6" class="text-center text-muted py-4">No documents
                                                    uploaded.
                                                </td>
                                            </tr>
                                        @endforelse
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
@endsection
