@extends('layout.app')
@section('content')
<div>
    <div class="page-title">
        <div class="row">
            <div class="col-6"><h4>Collections Desk</h4></div>
            <div class="col-6">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item"><a href="{{ route('dashboard') }}"><i data-feather="home"></i></a></li>
                    <li class="breadcrumb-item">Dispatch</li>
                    <li class="breadcrumb-item active">Collections Desk</li>
                </ol>
            </div>
        </div>
    </div>
</div>

<div class="container-fluid">
    <!-- Stats -->
    <div class="row mb-4">
        <div class="col-sm-4">
            <div class="card {{ $stloadsStats['needs_archive'] > 0 ? 'border-warning' : '' }}">
                <div class="card-body d-flex align-items-center">
                    <div class="rounded-circle bg-light-warning p-3 me-3"><i data-feather="alert-triangle" class="text-warning"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['needs_archive'] }}</h5>
                        <span class="text-muted f-12">Needs STLOADS Archive</span>
                    </div>
                </div>
            </div>
        </div>
        <div class="col-sm-4">
            <div class="card {{ $stloadsStats['sync_errors'] > 0 ? 'border-danger' : '' }}">
                <div class="card-body d-flex align-items-center">
                    <div class="rounded-circle bg-light-danger p-3 me-3"><i data-feather="zap" class="text-danger"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['sync_errors'] }}</h5>
                        <span class="text-muted f-12">Delivered-Still-Open Errors</span>
                    </div>
                </div>
            </div>
        </div>
        <div class="col-sm-4">
            <div class="card">
                <div class="card-body d-flex align-items-center">
                    <div class="rounded-circle bg-light-dark p-3 me-3"><i data-feather="archive" class="text-dark"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['closed'] }}</h5>
                        <span class="text-muted f-12">Closed / Archived</span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Collections-stage Loads -->
    <div class="card">
        <div class="card-header d-flex justify-content-between align-items-center">
            <h5 class="mb-0">Loads at Collections / Finance Stage</h5>
            <div>
                <a href="{{ route('stloads.sync-errors') }}" class="btn btn-outline-danger btn-sm me-2">
                    <i data-feather="zap" class="me-1" style="width:14px;height:14px;"></i> Sync Errors
                </a>
                <a href="{{ route('stloads.reconciliation') }}" class="btn btn-outline-warning btn-sm">
                    <i data-feather="git-pull-request" class="me-1" style="width:14px;height:14px;"></i> Reconciliation
                </a>
            </div>
        </div>
        <div class="card-body p-0">
            <div class="table-responsive">
                <table class="table table-hover mb-0">
                    <thead>
                        <tr>
                            <th>Load #</th>
                            <th>Title</th>
                            <th>Finance Status</th>
                            <th>STLOADS</th>
                            <th>Archive Status</th>
                        </tr>
                    </thead>
                    <tbody>
                        @forelse($legs as $leg)
                            @php $load = $leg->load_master; $ho = $load?->stloadsHandoff; @endphp
                            <tr>
                                <td><a href="{{ route('loads.view', $load) }}">{{ $load?->load_number ?? '—' }}</a></td>
                                <td>{{ $load?->title ?? '—' }}</td>
                                <td><span class="badge bg-dark">{{ $leg->status_master?->name ?? '—' }}</span></td>
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
                                        <span class="text-muted">—</span>
                                    @endif
                                </td>
                                <td>
                                    @if(!$ho)
                                        <span class="text-muted">No handoff</span>
                                    @elseif($ho->status === 'closed')
                                        <span class="badge bg-dark"><i data-feather="check" style="width:12px;height:12px;"></i> Archived</span>
                                    @elseif($ho->status === 'withdrawn')
                                        <span class="badge bg-secondary">Withdrawn — Ready to Close</span>
                                    @else
                                        <span class="badge bg-danger">
                                            <i data-feather="alert-circle" style="width:12px;height:12px;"></i> Still Active — Archive Required
                                        </span>
                                    @endif
                                </td>
                            </tr>
                        @empty
                            <tr><td colspan="5" class="text-center text-muted py-4">No loads at collections stage</td></tr>
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
