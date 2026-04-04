@extends('layout.app')
@section('content')
<div>
    <div class="page-title">
        <div class="row">
            <div class="col-6">
                <h4>STLOADS Operations</h4>
            </div>
            <div class="col-6">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item"><a href="{{ route('dashboard') }}"><i data-feather="home"></i></a></li>
                    <li class="breadcrumb-item active">STLOADS Operations</li>
                </ol>
            </div>
        </div>
    </div>
</div>

<div class="container-fluid">
    <!-- Sync Error Alert Banner -->
    @if($syncErrorCounts['total'] > 0)
    <div class="alert {{ $syncErrorCounts['critical'] > 0 ? 'alert-danger' : ($syncErrorCounts['error'] > 0 ? 'alert-warning' : 'alert-info') }} d-flex align-items-center justify-content-between mb-4" role="alert">
        <div>
            <i data-feather="alert-octagon" style="width:20px;height:20px;" class="me-2"></i>
            <strong>{{ $syncErrorCounts['total'] }} unresolved sync {{ $syncErrorCounts['total'] === 1 ? 'issue' : 'issues' }}</strong>
            @if($syncErrorCounts['critical'] > 0)
                &mdash; <span class="text-danger fw-bold">{{ $syncErrorCounts['critical'] }} critical</span>
            @endif
            @if($syncErrorCounts['error'] > 0)
                &mdash; {{ $syncErrorCounts['error'] }} {{ Str::plural('error', $syncErrorCounts['error']) }}
            @endif
            @if($syncErrorCounts['warning'] > 0)
                &mdash; {{ $syncErrorCounts['warning'] }} {{ Str::plural('warning', $syncErrorCounts['warning']) }}
            @endif
        </div>
        <a href="{{ route('stloads.sync-errors', ['resolved' => '0']) }}" class="btn btn-sm btn-outline-dark">
            View All Issues
        </a>
    </div>

    <!-- Recent Sync Errors -->
    @if($syncErrors->isNotEmpty())
    <div class="card mb-4 border-start border-4 {{ $syncErrorCounts['critical'] > 0 ? 'border-danger' : 'border-warning' }}">
        <div class="card-header card-no-border pb-0">
            <h6 class="mb-0">Recent Sync Issues</h6>
        </div>
        <div class="card-body pt-2">
            <div class="table-responsive">
                <table class="table table-sm align-middle text-nowrap mb-0" style="font-size: 0.82rem;">
                    <thead>
                        <tr>
                            <th>Severity</th>
                            <th>Class</th>
                            <th>Title</th>
                            <th>Handoff</th>
                            <th>When</th>
                            <th>Action</th>
                        </tr>
                    </thead>
                    <tbody>
                        @foreach($syncErrors as $err)
                        <tr>
                            <td>
                                @php
                                    $sevBadge = match($err->severity) {
                                        'critical' => 'bg-danger',
                                        'error'    => 'bg-warning text-dark',
                                        'warning'  => 'bg-info text-dark',
                                        default    => 'bg-light text-dark',
                                    };
                                @endphp
                                <span class="badge {{ $sevBadge }} rounded-pill">{{ ucfirst($err->severity) }}</span>
                            </td>
                            <td><code>{{ $err->error_class }}</code></td>
                            <td class="text-truncate" style="max-width: 300px;" title="{{ $err->title }}">{{ $err->title }}</td>
                            <td>
                                @if($err->handoff_id)
                                    <a href="{{ route('stloads.handoff.show', $err->handoff_id) }}">#{{ $err->handoff_id }}</a>
                                @else
                                    <span class="text-muted">—</span>
                                @endif
                            </td>
                            <td>{{ $err->created_at->diffForHumans() }}</td>
                            <td>
                                <form action="{{ route('stloads.sync-error.resolve', $err) }}" method="POST" class="d-inline">
                                    @csrf
                                    <button type="submit" class="btn btn-xs btn-outline-success" title="Mark resolved">
                                        <i data-feather="check" style="width:14px;height:14px;"></i>
                                    </button>
                                </form>
                            </td>
                        </tr>
                        @endforeach
                    </tbody>
                </table>
            </div>
        </div>
    </div>
    @endif
    @endif

    <!-- Status Summary Cards -->
    <div class="row g-3 mb-4">
        <div class="col-xl col-sm-4 col-6">
            <a href="{{ route('stloads.operations', ['status' => 'queued']) }}" class="text-decoration-none">
                <div class="card h-100 {{ $statusFilter === 'queued' ? 'border-primary border-2' : '' }}">
                    <div class="card-body text-center py-3">
                        <i data-feather="clock" class="text-warning mb-2" style="width:28px;height:28px;"></i>
                        <h4 class="mb-0">{{ $counts['queued'] }}</h4>
                        <span class="f-light small">Queued</span>
                    </div>
                </div>
            </a>
        </div>
        <div class="col-xl col-sm-4 col-6">
            <a href="{{ route('stloads.operations', ['status' => 'push_in_progress']) }}" class="text-decoration-none">
                <div class="card h-100 {{ $statusFilter === 'push_in_progress' ? 'border-primary border-2' : '' }}">
                    <div class="card-body text-center py-3">
                        <i data-feather="loader" class="text-info mb-2" style="width:28px;height:28px;"></i>
                        <h4 class="mb-0">{{ $counts['push_in_progress'] }}</h4>
                        <span class="f-light small">In Progress</span>
                    </div>
                </div>
            </a>
        </div>
        <div class="col-xl col-sm-4 col-6">
            <a href="{{ route('stloads.operations', ['status' => 'published']) }}" class="text-decoration-none">
                <div class="card h-100 {{ $statusFilter === 'published' ? 'border-primary border-2' : '' }}">
                    <div class="card-body text-center py-3">
                        <i data-feather="check-circle" class="text-success mb-2" style="width:28px;height:28px;"></i>
                        <h4 class="mb-0">{{ $counts['published'] }}</h4>
                        <span class="f-light small">Published</span>
                    </div>
                </div>
            </a>
        </div>
        <div class="col-xl col-sm-4 col-6">
            <a href="{{ route('stloads.operations', ['status' => 'push_failed']) }}" class="text-decoration-none">
                <div class="card h-100 {{ $statusFilter === 'push_failed' ? 'border-primary border-2' : '' }}">
                    <div class="card-body text-center py-3">
                        <i data-feather="alert-triangle" class="text-danger mb-2" style="width:28px;height:28px;"></i>
                        <h4 class="mb-0">{{ $counts['push_failed'] }}</h4>
                        <span class="f-light small">Failed</span>
                    </div>
                </div>
            </a>
        </div>
        <div class="col-xl col-sm-4 col-6">
            <a href="{{ route('stloads.operations', ['status' => 'requeue_required']) }}" class="text-decoration-none">
                <div class="card h-100 {{ $statusFilter === 'requeue_required' ? 'border-primary border-2' : '' }}">
                    <div class="card-body text-center py-3">
                        <i data-feather="refresh-cw" class="text-primary mb-2" style="width:28px;height:28px;"></i>
                        <h4 class="mb-0">{{ $counts['requeue_required'] }}</h4>
                        <span class="f-light small">Requeue</span>
                    </div>
                </div>
            </a>
        </div>
        <div class="col-xl col-sm-4 col-6">
            <a href="{{ route('stloads.operations', ['status' => 'withdrawn']) }}" class="text-decoration-none">
                <div class="card h-100 {{ $statusFilter === 'withdrawn' ? 'border-primary border-2' : '' }}">
                    <div class="card-body text-center py-3">
                        <i data-feather="x-circle" class="text-secondary mb-2" style="width:28px;height:28px;"></i>
                        <h4 class="mb-0">{{ $counts['withdrawn'] }}</h4>
                        <span class="f-light small">Withdrawn</span>
                    </div>
                </div>
            </a>
        </div>
        <div class="col-xl col-sm-4 col-6">
            <a href="{{ route('stloads.operations', ['status' => 'closed']) }}" class="text-decoration-none">
                <div class="card h-100 {{ $statusFilter === 'closed' ? 'border-primary border-2' : '' }}">
                    <div class="card-body text-center py-3">
                        <i data-feather="archive" class="text-dark mb-2" style="width:28px;height:28px;"></i>
                        <h4 class="mb-0">{{ $counts['closed'] }}</h4>
                        <span class="f-light small">Closed</span>
                    </div>
                </div>
            </a>
        </div>
    </div>

    <!-- Handoffs Table -->
    <div class="card">
        <div class="card-header card-no-border pb-0">
            <div class="d-flex justify-content-between align-items-center flex-wrap">
                <div>
                    <h5 class="mb-1">Handoff Records</h5>
                    <span class="f-light">
                        @if($statusFilter)
                            Showing <strong>{{ $statusFilter }}</strong> handoffs
                            <a href="{{ route('stloads.operations') }}" class="ms-2 small">&times; Clear filter</a>
                        @else
                            All handoffs
                        @endif
                    </span>
                </div>
            </div>
        </div>
        <div class="card-body pt-3">
            <div class="table-responsive">
                <table class="table table-striped align-middle text-nowrap" style="font-size: 0.875rem;">
                    <thead class="bg-white" style="position: sticky; top: 0; z-index: 2;">
                        <tr>
                            <th>#</th>
                            <th>TMS Load ID</th>
                            <th>Route</th>
                            <th>Mode</th>
                            <th>Equipment</th>
                            <th>Rate</th>
                            <th>Status</th>
                            <th>Load #</th>
                            <th>Retries</th>
                            <th>Pushed At</th>
                            <th>Action</th>
                        </tr>
                    </thead>
                    <tbody>
                        @forelse($handoffs as $h)
                            <tr>
                                <td>{{ $h->id }}</td>
                                <td class="fw-semibold">{{ $h->tms_load_id }}</td>
                                <td>
                                    <span data-bs-toggle="tooltip" title="{{ $h->pickup_address }}">
                                        {{ $h->pickup_city }}, {{ $h->pickup_state }}
                                    </span>
                                    <i data-feather="arrow-right" style="width:14px;height:14px;" class="mx-1 text-muted"></i>
                                    <span data-bs-toggle="tooltip" title="{{ $h->dropoff_address }}">
                                        {{ $h->dropoff_city }}, {{ $h->dropoff_state }}
                                    </span>
                                </td>
                                <td>{{ $h->freight_mode }}</td>
                                <td>{{ $h->equipment_type }}</td>
                                <td>${{ number_format($h->board_rate, 2) }}</td>
                                <td>
                                    @php
                                        $badge = match($h->status) {
                                            'queued'           => 'bg-warning text-dark',
                                            'push_in_progress' => 'bg-info',
                                            'published'        => 'bg-success',
                                            'push_failed'      => 'bg-danger',
                                            'requeue_required' => 'bg-primary',
                                            'withdrawn'        => 'bg-secondary',
                                            'closed'           => 'bg-dark',
                                            default            => 'bg-light text-dark',
                                        };
                                    @endphp
                                    <span class="badge rounded-pill {{ $badge }} p-2">
                                        {{ str_replace('_', ' ', ucfirst($h->status)) }}
                                    </span>
                                </td>
                                <td>
                                    @if($h->load)
                                        <a href="{{ route('loads.view', $h->load_id) }}">{{ $h->load->load_number }}</a>
                                    @else
                                        <span class="text-muted">—</span>
                                    @endif
                                </td>
                                <td>
                                    @if($h->retry_count > 0)
                                        <span class="badge bg-warning text-dark">{{ $h->retry_count }}</span>
                                    @else
                                        0
                                    @endif
                                </td>
                                <td>{{ $h->created_at->format('M d, Y H:i') }}</td>
                                <td>
                                    <a href="{{ route('stloads.handoff.show', $h->id) }}"
                                       class="btn btn-sm btn-outline-primary" data-bs-toggle="tooltip" title="View Details">
                                        <i data-feather="eye" style="width:14px;height:14px;"></i>
                                    </a>
                                </td>
                            </tr>
                        @empty
                            <tr>
                                <td colspan="11" class="text-center py-4 text-muted">
                                    No handoff records found.
                                </td>
                            </tr>
                        @endforelse
                    </tbody>
                </table>
            </div>

            <div class="mt-3">
                {{ $handoffs->appends(request()->query())->links() }}
            </div>
        </div>
    </div>
</div>
@endsection

@section('script')
<script>feather.replace();</script>
@endsection
