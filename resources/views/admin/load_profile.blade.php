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
                                    <div class="col-sm-4">
                                        <label class="form-label">Equipment</label>
                                        <input class="form-control" type="text" value="{{ $load->equipment->name }}"
                                            readonly>
                                    </div>
                                    <div class="col-sm-4">
                                        <label class="form-label">Commodity Type</label>
                                        <input class="form-control" type="text" value="{{ $load->commodity_type->name }}"
                                            readonly>
                                    </div>
                                    <div class="col-sm-4">
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
            <div class="card-options">
                <a class="card-options-collapse" href="#" data-bs-toggle="card-collapse">
                    <i class="fe fe-chevron-up"></i>
                </a>
                <a class="card-options-remove" href="#" data-bs-toggle="card-remove">
                    <i class="fe fe-x"></i>
                </a>
            </div>
        </div>

        <div class="card-body p-0">
            <div class="table-responsive">
                <div class="overflow-auto px-4">
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
                            return sprintf("%.{$decimals}f", $bytes / pow(1024, $factor)) . ' ' . $size[$factor];
                        }
                    @endphp

                    <table class="table table-striped align-middle text-nowrap" id="user-approval-table">
                        <div style="max-height: 500px; overflow: auto;">
                        <thead class="sticky-top bg-white z-index-sticky">
                            <tr>
                                <th>#</th>
                                <th>Document</th>
                                <th>File</th>
                                <th>Type</th>
                                <th>Size</th>
                                <th>Action</th>
                            </tr>
                        </thead>
                        <tbody>
                            @forelse ($load->loadDocuments as $i => $doc)
                                @php
                                    $exists = $doc->file_path && Storage::disk('public')->exists($doc->file_path);
                                    $url = $exists ? $doc->file_url : null;
                                    $ext = $doc->original_name
                                        ? strtolower(pathinfo($doc->original_name, PATHINFO_EXTENSION))
                                        : null;
                                    $mime = $doc->mime_type;
                                @endphp
                                <tr>
                                    <td>{{ $loop->iteration }}</td>

                                    <!-- Document Name -->
                                    <td>
                                        <div class="text-uppercase">
                                            {{ $doc->document_name ?? 'UNTITLED' }}
                                            @if (($doc->document_type ?? '') === 'blockchain')
                                                <span class="badge bg-primary ms-2">BLOCKCHAIN</span>
                                            @endif
                                        </div>
                                    </td>

                                    <!-- File -->
                                    <td>
                                        <div class="text-muted small">
                                            {{ $doc->original_name ?? basename($doc->file_path) }}
                                        </div>
                                    </td>

                                    <!-- Type -->
                                    <td class="text-nowrap">
                                        @if ($ext)
                                            {{ strtoupper($ext) }}
                                        @elseif($mime)
                                            {{ $mime }}
                                        @else
                                            —
                                        @endif
                                    </td>

                                    <!-- Size -->
                                    <td>{{ human_filesize($doc->file_size) }}</td>

                                    <!-- Action -->
                                    <td class="d-flex align-items-center gap-1">
                                        @if ($exists)
                                            {{-- Blockchain hash --}}
                                            @if (($doc->document_type ?? '') === 'blockchain' && $doc->hash)
                                                <button type="button" class="btn btn-sm btn-light"
                                                    data-bs-toggle="tooltip" data-bs-placement="top"
                                                    title="Hash: {{ $doc->hash }}">
                                                    Hash
                                                </button>
                                            @endif

                                            {{-- Preview --}}
                                            @if (in_array($mime, $imageMimes) || in_array($mime, $pdfMimes))
                                                <a href="{{ $url }}" target="_blank" rel="noopener"
                                                    class="btn btn-sm btn-secondary" data-bs-toggle="tooltip"
                                                    data-bs-placement="top" title="Preview">
                                                    Preview
                                                </a>
                                            @endif

                                            {{-- Download --}}
                                            <a href="{{ $url }}"
                                                download="{{ $doc->original_name ?? basename($doc->file_path) }}"
                                                class="btn btn-sm btn-primary" data-bs-toggle="tooltip"
                                                data-bs-placement="top" title="Download">
                                                Download
                                            </a>
                                        @else
                                            <span class="badge bg-danger">Missing</span>
                                        @endif
                                    </td>
                                </tr>
                            @empty
                                <tr>
                                    <td colspan="6" class="text-center text-muted py-4">No documents uploaded.</td>
                                </tr>
                            @endforelse
                        </tbody>
                        </div>
                    </table>
                </div>
            </div>
        </div>
    </div>
</div>

        </div>
    </div>
@endsection
