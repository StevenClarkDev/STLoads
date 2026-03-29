@extends('admin-layout.app')
@section('content')
    <div class="row">
        <div class="col-xl-12 mt-3">
            <div class="card mx-3">
                <div class="card-body">
                    <div class="row gy-4 px-3">
                        <!-- Left Column -->
                        <div class="col-xl-8">
                            <h5 class="mb-3">User Information</h5>
                            <div class="row g-3">
                                <div class="col-sm-6">
                                    <label class="form-label">Name</label>
                                    <input class="form-control" id="userName" type="text" value="{{ $user->name }}"
                                        readonly>
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
                                <div class="col-sm-6">
                                    <label class="form-label">Phone Number</label>
                                    <input class="form-control" type="text" value="{{ $user->phone_no ?? '—' }}" readonly>
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
                                                <use href="../assets/svg/icon-sprite.svg#profile-check"></use>
                                            </svg>
                                        </div>
                                    </div>
                                    <div class="social-details text-white text-center">
                                        <h5 class="mb-1 text-white">{{ $user->name }}</h5>
                                        <span class="text-light mb-4 d-block">{{ $user->email }}</span>
                                        @php
                                            $statusMap = [0=>'Onboarding',1=>'Approved',2=>'Rejected',3=>'Pending Review',4=>'Pending OTP',5=>'Needs Revision'];
                                            $statusColor = [0=>'secondary',1=>'success',2=>'danger',3=>'warning',4=>'info',5=>'warning'];
                                            $s = $user->status;
                                        @endphp
                                        <span class="badge bg-{{ $statusColor[$s] ?? 'secondary' }} px-3 py-2">
                                            {{ $statusMap[$s] ?? 'Unknown' }}
                                        </span>
                                        <div class="mt-3">
                                            <a href="{{ route('users.edit', $user->id) }}" class="btn btn-light btn-sm px-4">
                                                <i class="fa fa-edit"></i> Edit User
                                            </a>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div> <!-- end row -->
                </div>
            </div>
        </div>
        <div class="col-md-12">
            <div class="card p-4 mx-4">
                <div class="card-header py-2">
                    <h5 class="mb-0">Registration Details</h5>
                </div>
                <div class="card-body">
                    {{-- Broker / Generic --}}
                    @if ($user->hasRole('Broker'))
                        <div class="row g-3">
                            <div class="col-sm-4">
                                <label class="form-label">SSN #</label>
                                <input class="form-control" type="text" value="{{ $user->ssn_no ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">UCR / HCC #</label>
                                <input class="form-control" type="text" value="{{ $user->ucr_hcc_no ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">MC / CBSA / USDOT #</label>
                                <input class="form-control" type="text" value="{{ $user->mc_cbsa_usdot_no ?? '—' }}" readonly>
                            </div>
                        </div>
                    @endif

                    {{-- Carrier --}}
                    @if ($user->hasRole('Carrier'))
                        <h6 class="text-muted mb-2 mt-1">Identity</h6>
                        <div class="row g-3 mb-3">
                            <div class="col-sm-4">
                                <label class="form-label">Government ID #</label>
                                <input class="form-control" type="text" value="{{ $user->gov_id_number ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Nationality</label>
                                <input class="form-control" type="text" value="{{ $user->nationality ?? '—' }}" readonly>
                            </div>
                        </div>
                        <h6 class="text-muted mb-2">Driver License</h6>
                        <div class="row g-3 mb-3">
                            <div class="col-sm-4">
                                <label class="form-label">CDL Number</label>
                                <input class="form-control" type="text" value="{{ $user->cdl_number ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">CDL Class</label>
                                <input class="form-control" type="text" value="{{ $user->cdl_class ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">CDL Expiry</label>
                                <input class="form-control" type="text" value="{{ $user->cdl_expiry ?? '—' }}" readonly>
                            </div>
                        </div>
                        <h6 class="text-muted mb-2">Regulatory</h6>
                        <div class="row g-3 mb-3">
                            <div class="col-sm-3">
                                <label class="form-label">Country</label>
                                <input class="form-control" type="text" value="{{ $user->regulatory_country ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">USDOT #</label>
                                <input class="form-control" type="text" value="{{ $user->usdot_number ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">MC #</label>
                                <input class="form-control" type="text" value="{{ $user->mc_number ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">NTN / VAT #</label>
                                <input class="form-control" type="text" value="{{ $user->ntn ?? $user->vat_number ?? '—' }}" readonly>
                            </div>
                        </div>
                        <h6 class="text-muted mb-2">Insurance</h6>
                        <div class="row g-3 mb-3">
                            <div class="col-sm-4">
                                <label class="form-label">Insurer</label>
                                <input class="form-control" type="text" value="{{ $user->insurer_name ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Coverage Limits</label>
                                <input class="form-control" type="text" value="{{ $user->coverage_limits ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Policy Expiry</label>
                                <input class="form-control" type="text" value="{{ $user->insurance_expiry ?? '—' }}" readonly>
                            </div>
                        </div>
                        <h6 class="text-muted mb-2">Vehicle</h6>
                        <div class="row g-3 mb-3">
                            <div class="col-sm-3">
                                <label class="form-label">Reg #</label>
                                <input class="form-control" type="text" value="{{ $user->vehicle_reg ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">Make / Model</label>
                                <input class="form-control" type="text" value="{{ $user->vehicle_make_model ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-2">
                                <label class="form-label">Year</label>
                                <input class="form-control" type="text" value="{{ $user->vehicle_year ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-2">
                                <label class="form-label">Type</label>
                                <input class="form-control" type="text" value="{{ $user->vehicle_type ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-2">
                                <label class="form-label">Load Capacity</label>
                                <input class="form-control" type="text" value="{{ $user->load_capacity ?? '—' }}" readonly>
                            </div>
                        </div>
                        <h6 class="text-muted mb-2">Company (Optional)</h6>
                        <div class="row g-3">
                            <div class="col-sm-4">
                                <label class="form-label">Company Name</label>
                                <input class="form-control" type="text" value="{{ $user->company_name ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Reg #</label>
                                <input class="form-control" type="text" value="{{ $user->registration_number ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Tax ID</label>
                                <input class="form-control" type="text" value="{{ $user->tax_id ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Country of Incorporation</label>
                                <input class="form-control" type="text" value="{{ $user->country_of_incorporation ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Bank Account</label>
                                <input class="form-control" type="text" value="{{ $user->bank_account ?? '—' }}" readonly>
                            </div>
                        </div>
                    @endif

                    {{-- Shipper --}}
                    @if ($user->hasRole('Shipper'))
                        <div class="row g-3">
                            <div class="col-sm-4">
                                <label class="form-label">Nationality</label>
                                <input class="form-control" type="text" value="{{ $user->nationality ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Company Name</label>
                                <input class="form-control" type="text" value="{{ $user->company_name ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Business Registration #</label>
                                <input class="form-control" type="text" value="{{ $user->registration_number ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Tax ID</label>
                                <input class="form-control" type="text" value="{{ $user->tax_id ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Country of Incorporation</label>
                                <input class="form-control" type="text" value="{{ $user->country_of_incorporation ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Source of Funds</label>
                                <input class="form-control" type="text" value="{{ $user->source_of_funds ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Politically Exposed Person</label>
                                <input class="form-control" type="text" value="{{ $user->politically_exposed_person ? 'Yes' : 'No' }}" readonly>
                            </div>
                        </div>
                    @endif

                    {{-- Freight Forwarder --}}
                    @if ($user->hasRole('Freight Forwarder'))
                        <h6 class="text-muted mb-2 mt-1">Company</h6>
                        <div class="row g-3 mb-3">
                            <div class="col-sm-4">
                                <label class="form-label">Legal Name</label>
                                <input class="form-control" type="text" value="{{ $user->name }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Trade Name (DBA)</label>
                                <input class="form-control" type="text" value="{{ $user->trade_name ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Business Reg #</label>
                                <input class="form-control" type="text" value="{{ $user->registration_number ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Tax ID</label>
                                <input class="form-control" type="text" value="{{ $user->tax_id ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Country of Incorporation</label>
                                <input class="form-control" type="text" value="{{ $user->country_of_incorporation ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-4">
                                <label class="form-label">Incorporation Date</label>
                                <input class="form-control" type="text" value="{{ $user->incorporation_date ?? '—' }}" readonly>
                            </div>
                        </div>
                        <h6 class="text-muted mb-2">Director / UBO</h6>
                        <div class="row g-3 mb-3">
                            <div class="col-sm-3">
                                <label class="form-label">Director Name</label>
                                <input class="form-control" type="text" value="{{ $user->director_name ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">Director DOB</label>
                                <input class="form-control" type="text" value="{{ $user->director_dob ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">UBO Name</label>
                                <input class="form-control" type="text" value="{{ $user->ubo_name ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">UBO Nationality</label>
                                <input class="form-control" type="text" value="{{ $user->ubo_nationality ?? '—' }}" readonly>
                            </div>
                        </div>
                        <h6 class="text-muted mb-2">Licensing &amp; Insurance</h6>
                        <div class="row g-3 mb-3">
                            <div class="col-sm-3">
                                <label class="form-label">FMC License</label>
                                <input class="form-control" type="text" value="{{ $user->fmc_license ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">NVOCC Reg</label>
                                <input class="form-control" type="text" value="{{ $user->nvocc_reg ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">IATA Accreditation</label>
                                <input class="form-control" type="text" value="{{ $user->iata_accreditation ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">EORI #</label>
                                <input class="form-control" type="text" value="{{ $user->eori_number ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">Policy #</label>
                                <input class="form-control" type="text" value="{{ $user->policy_number ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">Coverage Limits</label>
                                <input class="form-control" type="text" value="{{ $user->coverage_limits ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">Insurance Expiry</label>
                                <input class="form-control" type="text" value="{{ $user->insurance_expiry ?? '—' }}" readonly>
                            </div>
                        </div>
                        <h6 class="text-muted mb-2">Operations</h6>
                        <div class="row g-3">
                            <div class="col-sm-3">
                                <label class="form-label">Transport Mode</label>
                                <input class="form-control" type="text" value="{{ $user->transport_modes ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">Countries Served</label>
                                <input class="form-control" type="text" value="{{ $user->countries_served ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">Years in Operation</label>
                                <input class="form-control" type="text" value="{{ $user->years_in_operation ?? '—' }}" readonly>
                            </div>
                            <div class="col-sm-3">
                                <label class="form-label">Annual Volume</label>
                                <input class="form-control" type="text" value="{{ $user->annual_volume ?? '—' }}" readonly>
                            </div>
                        </div>
                    @endif
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
                        <div class="overflow-auto px-4" style="max-height: 200px;">
                            @php
                                use Illuminate\Support\Facades\Storage;

                                $imageMimes = ['image/jpeg', 'image/png'];
                                $pdfMimes = ['application/pdf'];
                                $docxMimes = [
                                    'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
                                ];

                                if (!function_exists('human_filesize')) {
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
                                }
                            @endphp

                            <table class="table table-striped w-100 mb-0" id="user-approval-table">
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
                                    @forelse ($user->kycDocuments as $i => $doc)
                                        @php
                                            $exists = $doc->file_path && Storage::disk('public')->exists($doc->file_path);
                                            $url = $exists ? route('admin.serve-kyc-file', ['path' => $doc->file_path]) : null;
                                            $ext = $doc->original_name ? strtolower(pathinfo($doc->original_name, PATHINFO_EXTENSION)) : null;
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

                                                    {{-- Hash: only if blockchain hash exists --}}
                                                    @if (($doc->document_type ?? '') === 'blockchain' && $doc->hash)
                                                        <button type="button" class="btn btn-sm btn-light" data-bs-toggle="tooltip"
                                                            data-bs-placement="top" title="Hash: {{ $doc->hash }}">
                                                            Hash
                                                        </button>
                                                    @endif
                                                    {{-- Preview: only for images and PDFs --}}
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
                                            <td colspan="5" class="text-center text-muted py-4">No documents uploaded.</td>
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

    <script>
        document.addEventListener("DOMContentLoaded", function () {
            if (typeof feather !== "undefined") {
                feather.replace();
            }
            const tooltipTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'))
            tooltipTriggerList.map(function (tooltipTriggerEl) {
                return new bootstrap.Tooltip(tooltipTriggerEl)
            })
        });
    </script>
@endsection