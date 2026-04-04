@extends('layout.app')
@section('content')
<div>
    <div class="page-title">
        <div class="row">
            <div class="col-6"><h4>Quote Desk</h4></div>
            <div class="col-6">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item"><a href="{{ route('dashboard') }}"><i data-feather="home"></i></a></li>
                    <li class="breadcrumb-item">Dispatch</li>
                    <li class="breadcrumb-item active">Quote Desk</li>
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
                    <div class="rounded-circle bg-light-primary p-3 me-3"><i data-feather="target" class="text-primary"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['eligible'] }}</h5>
                        <span class="text-muted f-12">Eligible for STLOADS</span>
                    </div>
                </div>
            </div>
        </div>
        <div class="col-sm-4">
            <div class="card">
                <div class="card-body d-flex align-items-center">
                    <div class="rounded-circle bg-light-success p-3 me-3"><i data-feather="check-circle" class="text-success"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['published'] }}</h5>
                        <span class="text-muted f-12">Published to Board</span>
                    </div>
                </div>
            </div>
        </div>
        <div class="col-sm-4">
            <div class="card">
                <div class="card-body d-flex align-items-center">
                    <div class="rounded-circle bg-light-info p-3 me-3"><i data-feather="clock" class="text-info"></i></div>
                    <div>
                        <h5 class="mb-0">{{ $stloadsStats['queued'] }}</h5>
                        <span class="text-muted f-12">Queued for Push</span>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Quote-stage Loads -->
    <div class="card">
        <div class="card-header d-flex justify-content-between align-items-center">
            <h5 class="mb-0">Loads at Quote Stage</h5>
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
                            <th>Equipment</th>
                            <th>Weight</th>
                            <th>Status</th>
                            <th>STLOADS</th>
                            <th>Board Eligibility</th>
                        </tr>
                    </thead>
                    <tbody>
                        @forelse($legs as $leg)
                            @php $load = $leg->load_master; $ho = $load?->stloadsHandoff; @endphp
                            <tr>
                                <td>
                                    <a href="{{ route('loads.view', $load) }}">{{ $load?->load_number ?? '—' }}</a>
                                </td>
                                <td>{{ $load?->title ?? '—' }}</td>
                                <td>{{ $load?->equipment?->name ?? '—' }}</td>
                                <td>{{ $load?->weight ?? '—' }}</td>
                                <td><span class="badge bg-warning text-dark">{{ $leg->status_master?->name ?? 'New' }}</span></td>
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
                                        <span class="badge bg-outline-primary"><i data-feather="check" style="width:12px;height:12px;"></i> Eligible</span>
                                    @elseif($ho->status === 'published')
                                        <span class="badge bg-success">On Board</span>
                                    @else
                                        <span class="badge bg-secondary">{{ str_replace('_', ' ', ucfirst($ho->status)) }}</span>
                                    @endif
                                </td>
                            </tr>
                        @empty
                            <tr><td colspan="7" class="text-center text-muted py-4">No loads at quote stage</td></tr>
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
