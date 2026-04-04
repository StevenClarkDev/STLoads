@extends('layout.app')
@section('content')
<div>
    <div class="page-title">
        <div class="row">
            <div class="col-6">
                <h4>STLOADS Sync Errors</h4>
            </div>
            <div class="col-6">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item"><a href="{{ route('dashboard') }}"><i data-feather="home"></i></a></li>
                    <li class="breadcrumb-item"><a href="{{ route('stloads.operations') }}">STLOADS Operations</a></li>
                    <li class="breadcrumb-item active">Sync Errors</li>
                </ol>
            </div>
        </div>
    </div>
</div>

<div class="container-fluid">
    <!-- Filter bar -->
    <div class="card mb-3">
        <div class="card-body py-2">
            <form class="d-flex flex-wrap gap-2 align-items-center">
                <span class="fw-semibold small me-2">Filters:</span>

                <a href="{{ route('stloads.sync-errors', ['resolved' => '0']) }}"
                   class="btn btn-sm {{ request('resolved') === '0' ? 'btn-primary' : 'btn-outline-primary' }}">
                    Unresolved
                </a>
                <a href="{{ route('stloads.sync-errors', ['resolved' => '1']) }}"
                   class="btn btn-sm {{ request('resolved') === '1' ? 'btn-success' : 'btn-outline-success' }}">
                    Resolved
                </a>
                <a href="{{ route('stloads.sync-errors') }}"
                   class="btn btn-sm {{ !request('resolved') && !request('severity') ? 'btn-dark' : 'btn-outline-dark' }}">
                    All
                </a>

                <span class="mx-2 text-muted">|</span>

                @foreach(['critical', 'error', 'warning', 'info'] as $sev)
                    <a href="{{ route('stloads.sync-errors', array_merge(request()->only('resolved'), ['severity' => $sev])) }}"
                       class="btn btn-sm {{ request('severity') === $sev ? 'btn-secondary' : 'btn-outline-secondary' }}">
                        {{ ucfirst($sev) }}
                    </a>
                @endforeach
            </form>
        </div>
    </div>

    <!-- Errors Table -->
    <div class="card">
        <div class="card-body">
            <div class="table-responsive">
                <table class="table table-striped align-middle text-nowrap" style="font-size: 0.875rem;">
                    <thead class="bg-white" style="position: sticky; top: 0; z-index: 2;">
                        <tr>
                            <th>#</th>
                            <th>Severity</th>
                            <th>Class</th>
                            <th>Title</th>
                            <th>Handoff</th>
                            <th>Source</th>
                            <th>Created</th>
                            <th>Status</th>
                            <th>Action</th>
                        </tr>
                    </thead>
                    <tbody>
                        @forelse($errors as $err)
                            <tr class="{{ $err->resolved ? 'opacity-50' : '' }}">
                                <td>{{ $err->id }}</td>
                                <td>
                                    @php
                                        $sevBadge = match($err->severity) {
                                            'critical' => 'bg-danger',
                                            'error'    => 'bg-warning text-dark',
                                            'warning'  => 'bg-info text-dark',
                                            default    => 'bg-light text-dark',
                                        };
                                    @endphp
                                    <span class="badge {{ $sevBadge }} rounded-pill p-2">{{ ucfirst($err->severity) }}</span>
                                </td>
                                <td><code>{{ $err->error_class }}</code></td>
                                <td class="text-wrap" style="max-width: 350px;">
                                    <strong>{{ $err->title }}</strong>
                                    @if($err->detail)
                                        <br><span class="text-muted small">{{ Str::limit($err->detail, 120) }}</span>
                                    @endif
                                </td>
                                <td>
                                    @if($err->handoff_id)
                                        <a href="{{ route('stloads.handoff.show', $err->handoff_id) }}">#{{ $err->handoff_id }}</a>
                                    @else
                                        <span class="text-muted">—</span>
                                    @endif
                                </td>
                                <td>{{ $err->source_module ?? '—' }}</td>
                                <td>{{ $err->created_at->format('M d, H:i') }}</td>
                                <td>
                                    @if($err->resolved)
                                        <span class="badge bg-success rounded-pill p-2">Resolved</span>
                                        <br><span class="text-muted small">{{ $err->resolved_by }} &middot; {{ $err->resolved_at->diffForHumans() }}</span>
                                        @if($err->resolution_note)
                                            <br><span class="text-muted small fst-italic">{{ $err->resolution_note }}</span>
                                        @endif
                                    @else
                                        <span class="badge bg-danger rounded-pill p-2">Open</span>
                                    @endif
                                </td>
                                <td>
                                    @unless($err->resolved)
                                        <form action="{{ route('stloads.sync-error.resolve', $err) }}" method="POST" class="d-inline">
                                            @csrf
                                            <div class="input-group input-group-sm" style="width: 200px;">
                                                <input type="text" name="resolution_note" class="form-control form-control-sm" placeholder="Note (optional)">
                                                <button type="submit" class="btn btn-outline-success btn-sm" title="Resolve">
                                                    <i data-feather="check" style="width:14px;height:14px;"></i>
                                                </button>
                                            </div>
                                        </form>
                                    @endunless
                                </td>
                            </tr>
                        @empty
                            <tr>
                                <td colspan="9" class="text-center text-muted py-4">
                                    No sync errors found.
                                </td>
                            </tr>
                        @endforelse
                    </tbody>
                </table>
            </div>

            <div class="mt-3">
                {{ $errors->withQueryString()->links() }}
            </div>
        </div>
    </div>
</div>
@endsection
