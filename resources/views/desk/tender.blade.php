@extends('layout.app')
@section('content')
<div>
    <div class="page-title">
        <div class="row">
            <div class="col-6"><h4>Tender Desk</h4></div>
            <div class="col-6">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item"><a href="{{ route('dashboard') }}"><i data-feather="home"></i></a></li>
                    <li class="breadcrumb-item">Dispatch</li>
                    <li class="breadcrumb-item active">Tender Desk</li>
                </ol>
            </div>
        </div>
    </div>
</div>

<div class="container-fluid">
    <!-- STLOADS Stats -->
    <div class="row mb-4">
        <div class="col-sm-4">
            <div class="card">
                <div class="card-body d-flex align-items-center">
                    <div class="rounded-circle bg-light-success p-3 me-3"><i data-feather="check-circle" class="text-success"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['published'] }}</h5>
                        <span class="text-muted f-12">Board-Exposed</span>
                    </div>
                </div>
            </div>
        </div>
        <div class="col-sm-4">
            <div class="card">
                <div class="card-body d-flex align-items-center">
                    <div class="rounded-circle bg-light-danger p-3 me-3"><i data-feather="alert-circle" class="text-danger"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['push_failed'] }}</h5>
                        <span class="text-muted f-12">Push Failed</span>
                    </div>
                </div>
            </div>
        </div>
        <div class="col-sm-4">
            <div class="card">
                <div class="card-body d-flex align-items-center">
                    <div class="rounded-circle bg-light-secondary p-3 me-3"><i data-feather="x-circle" class="text-secondary"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['withdrawn'] }}</h5>
                        <span class="text-muted f-12">Withdrawn</span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Tender-stage Loads -->
    <div class="card">
        <div class="card-header d-flex justify-content-between align-items-center">
            <h5 class="mb-0">Loads at Tender Stage</h5>
            <a href="{{ route('stloads.operations') }}" class="btn btn-outline-primary btn-sm">
                <i data-feather="radio" class="me-1" style="width:14px;height:14px;"></i> STLOADS Operations
            </a>
        </div>
        <div class="card-body p-0">
            <div class="table-responsive">
                <table class="table table-hover mb-0">
                    <thead>
                        <tr>
                            <th>Load #</th>
                            <th>Title</th>
                            <th>Carrier</th>
                            <th>Leg Status</th>
                            <th>STLOADS</th>
                            <th>Duplicate Risk</th>
                        </tr>
                    </thead>
                    <tbody>
                        @forelse($legs as $leg)
                            @php $load = $leg->load_master; $ho = $load?->stloadsHandoff; @endphp
                            <tr>
                                <td><a href="{{ route('loads.view', $load) }}">{{ $load?->load_number ?? '—' }}</a></td>
                                <td>{{ $load?->title ?? '—' }}</td>
                                <td>{{ $leg->carrier?->name ?? '<span class="text-muted">Unassigned</span>' }}</td>
                                <td><span class="badge bg-warning text-dark">{{ $leg->status_master?->name ?? '—' }}</span></td>
                                <td>
                                    @if($ho)
                                        <span class="badge {{ match($ho->status) {
                                            'published' => 'bg-success',
                                            'push_failed', 'requeue_required' => 'bg-danger',
                                            'queued', 'push_in_progress' => 'bg-info',
                                            'withdrawn' => 'bg-secondary',
                                            'closed' => 'bg-dark',
                                            default => 'bg-light text-dark',
                                        } }}">{{ str_replace('_', ' ', ucfirst($ho->status)) }}</span>
                                    @else
                                        <span class="text-muted">Not on STLOADS</span>
                                    @endif
                                </td>
                                <td>
                                    @if($ho && $ho->status === 'published' && $leg->booked_carrier_id)
                                        <span class="badge bg-warning text-dark" data-bs-toggle="tooltip" title="Load is booked with a carrier but still published on STLOADS board">
                                            <i data-feather="alert-triangle" style="width:12px;height:12px;"></i> Carrier Assigned
                                        </span>
                                    @else
                                        <span class="text-muted">—</span>
                                    @endif
                                </td>
                            </tr>
                        @empty
                            <tr><td colspan="6" class="text-center text-muted py-4">No loads at tender stage</td></tr>
                        @endforelse
                    </tbody>
                </table>
            </div>
        </div>
        @if($legs->hasPages())
            <div class="card-footer">{{ $legs->links() }}</div>
        @endif
    </div>
</div>
@endsection
