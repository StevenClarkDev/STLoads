@extends('layout.app')
@section('content')
<div>
    <div class="page-title">
        <div class="row">
            <div class="col-6">
                <h4>STLOADS Reconciliation</h4>
            </div>
            <div class="col-6">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item"><a href="{{ route('dashboard') }}"><i data-feather="home"></i></a></li>
                    <li class="breadcrumb-item"><a href="{{ route('stloads.operations') }}">STLOADS Operations</a></li>
                    <li class="breadcrumb-item active">Reconciliation</li>
                </ol>
            </div>
        </div>
    </div>
</div>

<div class="container-fluid">
    @if(session('success'))
        <div class="alert alert-success alert-dismissible fade show" role="alert">
            {{ session('success') }}
            <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
        </div>
    @endif

    <!-- Mismatch Overview Cards -->
    <div class="row g-3 mb-4">
        <div class="col-xl-2 col-sm-4 col-6">
            <div class="card h-100">
                <div class="card-body text-center py-3">
                    <i data-feather="radio" class="text-success mb-2" style="width:24px;height:24px;"></i>
                    <h4 class="mb-0">{{ $mismatchCounts['total_published'] }}</h4>
                    <span class="f-light small">Published</span>
                </div>
            </div>
        </div>
        <div class="col-xl-2 col-sm-4 col-6">
            <div class="card h-100 {{ $mismatchCounts['tms_cancelled'] > 0 ? 'border-danger border-2' : '' }}">
                <div class="card-body text-center py-3">
                    <i data-feather="x-octagon" class="text-danger mb-2" style="width:24px;height:24px;"></i>
                    <h4 class="mb-0">{{ $mismatchCounts['tms_cancelled'] }}</h4>
                    <span class="f-light small">TMS Cancelled</span>
                    <br><span class="text-muted" style="font-size:0.7rem;">still published</span>
                </div>
            </div>
        </div>
        <div class="col-xl-2 col-sm-4 col-6">
            <div class="card h-100 {{ $mismatchCounts['tms_delivered'] > 0 ? 'border-warning border-2' : '' }}">
                <div class="card-body text-center py-3">
                    <i data-feather="truck" class="text-warning mb-2" style="width:24px;height:24px;"></i>
                    <h4 class="mb-0">{{ $mismatchCounts['tms_delivered'] }}</h4>
                    <span class="f-light small">TMS Delivered</span>
                    <br><span class="text-muted" style="font-size:0.7rem;">still published</span>
                </div>
            </div>
        </div>
        <div class="col-xl-2 col-sm-4 col-6">
            <div class="card h-100 {{ $mismatchCounts['tms_invoiced'] > 0 ? 'border-info border-2' : '' }}">
                <div class="card-body text-center py-3">
                    <i data-feather="file-text" class="text-info mb-2" style="width:24px;height:24px;"></i>
                    <h4 class="mb-0">{{ $mismatchCounts['tms_invoiced'] }}</h4>
                    <span class="f-light small">TMS Invoiced/Settled</span>
                    <br><span class="text-muted" style="font-size:0.7rem;">still published</span>
                </div>
            </div>
        </div>
        <div class="col-xl-2 col-sm-4 col-6">
            <div class="card h-100">
                <div class="card-body text-center py-3">
                    <i data-feather="help-circle" class="text-secondary mb-2" style="width:24px;height:24px;"></i>
                    <h4 class="mb-0">{{ $mismatchCounts['no_tms_status'] }}</h4>
                    <span class="f-light small">No TMS Status</span>
                    <br><span class="text-muted" style="font-size:0.7rem;">never updated</span>
                </div>
            </div>
        </div>
        <div class="col-xl-2 col-sm-4 col-6">
            <div class="card h-100 {{ $mismatchCounts['stale_30d'] > 0 ? 'border-secondary border-2' : '' }}">
                <div class="card-body text-center py-3">
                    <i data-feather="clock" class="text-muted mb-2" style="width:24px;height:24px;"></i>
                    <h4 class="mb-0">{{ $mismatchCounts['stale_30d'] }}</h4>
                    <span class="f-light small">Stale (30d+)</span>
                    <br><span class="text-muted" style="font-size:0.7rem;">no webhook activity</span>
                </div>
            </div>
        </div>
    </div>

    <!-- Action Bar -->
    <div class="d-flex justify-content-between align-items-center mb-4">
        <div>
            <a href="{{ route('stloads.operations') }}" class="btn btn-outline-primary btn-sm me-2">
                <i data-feather="arrow-left" style="width:14px;height:14px;"></i> Operations
            </a>
            <a href="{{ route('stloads.sync-errors', ['resolved' => '0']) }}" class="btn btn-outline-warning btn-sm me-2">
                Sync Errors
            </a>
        </div>
        <form action="{{ route('stloads.reconciliation.scan') }}" method="POST" class="d-inline">
            @csrf
            <button type="submit" class="btn btn-primary btn-sm" onclick="return confirm('Run reconciliation scan? This will auto-fix detected mismatches.')">
                <i data-feather="refresh-cw" style="width:14px;height:14px;"></i> Run Reconciliation Scan
            </button>
        </form>
    </div>

    <div class="row">
        <!-- Error Breakdown -->
        <div class="col-xl-4 mb-4">
            <div class="card h-100">
                <div class="card-header">
                    <h6 class="mb-0">Unresolved Sync Errors by Class</h6>
                </div>
                <div class="card-body p-0">
                    @if($errorBreakdown->isEmpty())
                        <div class="text-center text-muted py-4">
                            <i data-feather="check-circle" style="width:32px;height:32px;" class="mb-2 text-success"></i>
                            <div>No unresolved errors</div>
                        </div>
                    @else
                        <table class="table table-sm mb-0" style="font-size: 0.82rem;">
                            <thead>
                                <tr>
                                    <th>Error Class</th>
                                    <th>Severity</th>
                                    <th>Count</th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach($errorBreakdown as $errorClass => $items)
                                    @foreach($items as $item)
                                        <tr>
                                            <td><code>{{ $errorClass }}</code></td>
                                            <td>
                                                @php
                                                    $sevBadge = match($item->severity) {
                                                        'critical' => 'bg-danger',
                                                        'error'    => 'bg-warning text-dark',
                                                        'warning'  => 'bg-info text-dark',
                                                        default    => 'bg-light text-dark',
                                                    };
                                                @endphp
                                                <span class="badge {{ $sevBadge }}">{{ ucfirst($item->severity) }}</span>
                                            </td>
                                            <td class="fw-semibold">{{ $item->cnt }}</td>
                                        </tr>
                                    @endforeach
                                @endforeach
                            </tbody>
                        </table>
                    @endif
                </div>
            </div>
        </div>

        <!-- Reconciliation Log -->
        <div class="col-xl-8 mb-4">
            <div class="card h-100">
                <div class="card-header card-no-border pb-0">
                    <div class="d-flex justify-content-between align-items-center flex-wrap">
                        <h6 class="mb-0">Reconciliation Log</h6>
                        <div class="d-flex gap-1 flex-wrap">
                            <a href="{{ route('stloads.reconciliation') }}"
                               class="btn btn-xs {{ !$actionFilter ? 'btn-dark' : 'btn-outline-dark' }}">All</a>
                            @foreach(['status_update', 'auto_withdraw', 'auto_close', 'auto_archive', 'rate_update', 'mismatch_detected', 'force_sync'] as $act)
                                <a href="{{ route('stloads.reconciliation', ['action' => $act]) }}"
                                   class="btn btn-xs {{ $actionFilter === $act ? 'btn-dark' : 'btn-outline-dark' }}">
                                    {{ str_replace('_', ' ', ucfirst($act)) }}
                                </a>
                            @endforeach
                        </div>
                    </div>
                </div>
                <div class="card-body pt-2">
                    <div class="table-responsive">
                        <table class="table table-sm align-middle text-nowrap" style="font-size: 0.82rem;">
                            <thead>
                                <tr>
                                    <th>#</th>
                                    <th>Action</th>
                                    <th>Handoff</th>
                                    <th>TMS Status</th>
                                    <th>STLOADS Status</th>
                                    <th>Detail</th>
                                    <th>By</th>
                                    <th>When</th>
                                </tr>
                            </thead>
                            <tbody>
                                @forelse($logs as $log)
                                    <tr>
                                        <td>{{ $log->id }}</td>
                                        <td>
                                            @php
                                                $actionBadge = match($log->action) {
                                                    'status_update'     => 'bg-info text-dark',
                                                    'auto_withdraw'     => 'bg-warning text-dark',
                                                    'auto_close'        => 'bg-secondary',
                                                    'auto_archive'      => 'bg-dark',
                                                    'rate_update'       => 'bg-primary',
                                                    'mismatch_detected' => 'bg-danger',
                                                    'force_sync'        => 'bg-success',
                                                    default             => 'bg-light text-dark',
                                                };
                                            @endphp
                                            <span class="badge {{ $actionBadge }}">{{ str_replace('_', ' ', $log->action) }}</span>
                                        </td>
                                        <td>
                                            @if($log->handoff)
                                                <a href="{{ route('stloads.handoff.show', $log->handoff_id) }}">
                                                    #{{ $log->handoff_id }}
                                                </a>
                                                <span class="text-muted small">({{ $log->handoff->tms_load_id }})</span>
                                            @else
                                                —
                                            @endif
                                        </td>
                                        <td>
                                            @if($log->tms_status_from || $log->tms_status_to)
                                                <span class="text-muted">{{ $log->tms_status_from ?? '—' }}</span>
                                                <i data-feather="arrow-right" style="width:12px;height:12px;" class="mx-1 text-muted"></i>
                                                <span class="fw-semibold">{{ $log->tms_status_to ?? '—' }}</span>
                                            @else
                                                —
                                            @endif
                                        </td>
                                        <td>
                                            @if($log->stloads_status_from || $log->stloads_status_to)
                                                <span class="text-muted">{{ $log->stloads_status_from ?? '—' }}</span>
                                                @if($log->stloads_status_from !== $log->stloads_status_to)
                                                    <i data-feather="arrow-right" style="width:12px;height:12px;" class="mx-1 text-muted"></i>
                                                    <span class="fw-semibold">{{ $log->stloads_status_to ?? '—' }}</span>
                                                @endif
                                            @else
                                                —
                                            @endif
                                        </td>
                                        <td class="text-truncate" style="max-width: 220px;" title="{{ $log->detail }}">
                                            {{ $log->detail ?? '—' }}
                                        </td>
                                        <td>{{ $log->triggered_by ?? '—' }}</td>
                                        <td>{{ $log->created_at->format('M d H:i') }}</td>
                                    </tr>
                                @empty
                                    <tr>
                                        <td colspan="8" class="text-center text-muted py-4">No reconciliation events yet.</td>
                                    </tr>
                                @endforelse
                            </tbody>
                        </table>
                    </div>
                    <div class="mt-2">
                        {{ $logs->withQueryString()->links() }}
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
@endsection

@section('script')
<script>feather.replace();</script>
@endsection
