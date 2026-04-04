@extends('layout.app')
@section('content')
<div>
    <div class="page-title">
        <div class="row">
            <div class="col-6"><h4>Closeout Desk</h4></div>
            <div class="col-6">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item"><a href="{{ route('dashboard') }}"><i data-feather="home"></i></a></li>
                    <li class="breadcrumb-item">Dispatch</li>
                    <li class="breadcrumb-item active">Closeout Desk</li>
                </ol>
            </div>
        </div>
    </div>
</div>

<div class="container-fluid">
    <!-- Stats -->
    <div class="row mb-4">
        <div class="col-sm-4">
            <div class="card {{ $stloadsStats['still_live'] > 0 ? 'border-danger' : '' }}">
                <div class="card-body d-flex align-items-center">
                    <div class="rounded-circle bg-light-danger p-3 me-3"><i data-feather="alert-triangle" class="text-danger"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['still_live'] }}</h5>
                        <span class="text-muted f-12">Still Live on STLOADS</span>
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

    <!-- Closeout-stage Loads -->
    <div class="card">
        <div class="card-header d-flex justify-content-between align-items-center">
            <h5 class="mb-0">Loads at Closeout Stage</h5>
            <div>
                <a href="{{ route('stloads.reconciliation') }}" class="btn btn-outline-warning btn-sm me-2">
                    <i data-feather="git-pull-request" class="me-1" style="width:14px;height:14px;"></i> Reconciliation
                </a>
                <a href="{{ route('stloads.operations') }}" class="btn btn-outline-primary btn-sm">
                    <i data-feather="radio" class="me-1" style="width:14px;height:14px;"></i> STLOADS Operations
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
                            <th>Delivery Status</th>
                            <th>STLOADS</th>
                            <th>Action Needed</th>
                        </tr>
                    </thead>
                    <tbody>
                        @forelse($legs as $leg)
                            @php $load = $leg->load_master; $ho = $load?->stloadsHandoff; @endphp
                            <tr>
                                <td><a href="{{ route('loads.view', $load) }}">{{ $load?->load_number ?? '—' }}</a></td>
                                <td>{{ $load?->title ?? '—' }}</td>
                                <td><span class="badge bg-success">{{ $leg->status_master?->name ?? '—' }}</span></td>
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
                                    @if($ho && in_array($ho->status, ['published', 'queued', 'push_in_progress']))
                                        <span class="badge bg-danger">
                                            <i data-feather="alert-circle" style="width:12px;height:12px;"></i> Needs Withdraw/Close
                                        </span>
                                    @elseif($ho && $ho->status === 'withdrawn')
                                        <span class="badge bg-warning text-dark">Needs Archive</span>
                                    @elseif($ho && $ho->status === 'closed')
                                        <span class="badge bg-dark"><i data-feather="check" style="width:12px;height:12px;"></i> Archived</span>
                                    @else
                                        <span class="text-muted">—</span>
                                    @endif
                                </td>
                            </tr>
                        @empty
                            <tr><td colspan="5" class="text-center text-muted py-4">No loads at closeout stage</td></tr>
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
