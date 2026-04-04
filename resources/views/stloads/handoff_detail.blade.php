@extends('layout.app')
@section('content')
<div>
    <div class="page-title">
        <div class="row">
            <div class="col-6">
                <h4>Handoff #{{ $handoff->id }}</h4>
            </div>
            <div class="col-6">
                <ol class="breadcrumb">
                    <li class="breadcrumb-item"><a href="{{ route('dashboard') }}"><i data-feather="home"></i></a></li>
                    <li class="breadcrumb-item"><a href="{{ route('stloads.operations') }}">STLOADS Operations</a></li>
                    <li class="breadcrumb-item active">Handoff #{{ $handoff->id }}</li>
                </ol>
            </div>
        </div>
    </div>
</div>

<div class="container-fluid">
    <div class="row">
        <!-- Left Column: Handoff Info -->
        <div class="col-xl-8">
            <!-- Status + Identity -->
            <div class="card mb-4">
                <div class="card-header d-flex justify-content-between align-items-center">
                    <h5 class="mb-0">Handoff Status</h5>
                    @php
                        $badge = match($handoff->status) {
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
                    <span class="badge {{ $badge }} p-2 fs-6">{{ str_replace('_', ' ', ucfirst($handoff->status)) }}</span>
                </div>
                <div class="card-body">
                    <div class="row g-3">
                        <div class="col-md-4">
                            <label class="f-light small">TMS Load ID</label>
                            <div class="fw-semibold">{{ $handoff->tms_load_id }}</div>
                        </div>
                        <div class="col-md-4">
                            <label class="f-light small">Tenant</label>
                            <div>{{ $handoff->tenant_id }}</div>
                        </div>
                        <div class="col-md-4">
                            <label class="f-light small">External Handoff ID</label>
                            <div>{{ $handoff->external_handoff_id ?? '—' }}</div>
                        </div>
                        <div class="col-md-4">
                            <label class="f-light small">STLoads Load #</label>
                            <div>
                                @if($handoff->load)
                                    <a href="{{ route('loads.view', $handoff->load_id) }}">{{ $handoff->load->load_number }}</a>
                                @else
                                    <span class="text-muted">Not materialized</span>
                                @endif
                            </div>
                        </div>
                        <div class="col-md-4">
                            <label class="f-light small">Pushed By</label>
                            <div>{{ $handoff->pushed_by ?? '—' }}</div>
                        </div>
                        <div class="col-md-4">
                            <label class="f-light small">Source Module</label>
                            <div>{{ $handoff->source_module ?? '—' }}</div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Freight Details -->
            <div class="card mb-4">
                <div class="card-header">
                    <h5 class="mb-0">Freight Details</h5>
                </div>
                <div class="card-body">
                    <div class="row g-3">
                        <div class="col-md-3">
                            <label class="f-light small">Party Type</label>
                            <div>{{ ucfirst(str_replace('_', ' ', $handoff->party_type)) }}</div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Freight Mode</label>
                            <div>{{ $handoff->freight_mode }}</div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Equipment</label>
                            <div>{{ $handoff->equipment_type }}</div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Commodity</label>
                            <div>{{ $handoff->commodity_description ?? '—' }}</div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Weight</label>
                            <div>{{ number_format($handoff->weight, 2) }} {{ $handoff->weight_unit }}</div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Pieces</label>
                            <div>{{ $handoff->piece_count ?? '—' }}</div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Hazardous</label>
                            <div>{!! $handoff->is_hazardous ? '<span class="badge bg-danger">Yes</span>' : 'No' !!}</div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Bid Type</label>
                            <div>{{ $handoff->bid_type }}</div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Route / Locations -->
            <div class="card mb-4">
                <div class="card-header">
                    <h5 class="mb-0">Route</h5>
                </div>
                <div class="card-body">
                    <div class="row">
                        <div class="col-md-6">
                            <h6 class="text-success"><i data-feather="map-pin" style="width:16px;height:16px;"></i> Pickup</h6>
                            <div class="ps-3">
                                <div>{{ $handoff->pickup_address }}</div>
                                <div>{{ $handoff->pickup_city }}, {{ $handoff->pickup_state }} {{ $handoff->pickup_zip }}</div>
                                <div>{{ $handoff->pickup_country }}</div>
                                <div class="mt-2 small text-muted">
                                    <strong>Window:</strong>
                                    {{ $handoff->pickup_window_start?->format('M d, Y H:i') }}
                                    @if($handoff->pickup_window_end)
                                        — {{ $handoff->pickup_window_end->format('H:i') }}
                                    @endif
                                </div>
                                @if($handoff->pickup_instructions)
                                    <div class="mt-1 small"><strong>Instructions:</strong> {{ $handoff->pickup_instructions }}</div>
                                @endif
                                @if($handoff->pickup_appointment_ref)
                                    <div class="small"><strong>Appt Ref:</strong> {{ $handoff->pickup_appointment_ref }}</div>
                                @endif
                            </div>
                        </div>
                        <div class="col-md-6">
                            <h6 class="text-danger"><i data-feather="map-pin" style="width:16px;height:16px;"></i> Dropoff</h6>
                            <div class="ps-3">
                                <div>{{ $handoff->dropoff_address }}</div>
                                <div>{{ $handoff->dropoff_city }}, {{ $handoff->dropoff_state }} {{ $handoff->dropoff_zip }}</div>
                                <div>{{ $handoff->dropoff_country }}</div>
                                <div class="mt-2 small text-muted">
                                    <strong>Window:</strong>
                                    {{ $handoff->dropoff_window_start?->format('M d, Y H:i') }}
                                    @if($handoff->dropoff_window_end)
                                        — {{ $handoff->dropoff_window_end->format('H:i') }}
                                    @endif
                                </div>
                                @if($handoff->dropoff_instructions)
                                    <div class="mt-1 small"><strong>Instructions:</strong> {{ $handoff->dropoff_instructions }}</div>
                                @endif
                                @if($handoff->dropoff_appointment_ref)
                                    <div class="small"><strong>Appt Ref:</strong> {{ $handoff->dropoff_appointment_ref }}</div>
                                @endif
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Pricing + Compliance -->
            <div class="card mb-4">
                <div class="card-header">
                    <h5 class="mb-0">Pricing & Compliance</h5>
                </div>
                <div class="card-body">
                    <div class="row g-3">
                        <div class="col-md-3">
                            <label class="f-light small">Board Rate</label>
                            <div class="fw-semibold fs-5">${{ number_format($handoff->board_rate, 2) }} {{ $handoff->rate_currency }}</div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Compliance</label>
                            <div>
                                @if($handoff->compliance_passed)
                                    <span class="badge bg-success">Passed</span>
                                @elseif($handoff->compliance_passed === false)
                                    <span class="badge bg-danger">Failed</span>
                                @else
                                    <span class="text-muted">—</span>
                                @endif
                            </div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Retry Count</label>
                            <div>{{ $handoff->retry_count ?? 0 }}</div>
                        </div>
                        <div class="col-md-3">
                            <label class="f-light small">Last Push Result</label>
                            <div>{{ $handoff->last_push_result ?? '—' }}</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Right Column: Timeline + Timestamps -->
        <div class="col-xl-4">
            <!-- Key Timestamps -->
            <div class="card mb-4">
                <div class="card-header">
                    <h5 class="mb-0">Timeline</h5>
                </div>
                <div class="card-body">
                    <ul class="list-unstyled">
                        <li class="d-flex align-items-start mb-3">
                            <i data-feather="plus-circle" class="text-info me-2 mt-1" style="width:16px;height:16px;"></i>
                            <div>
                                <div class="small fw-semibold">Created</div>
                                <div class="small text-muted">{{ $handoff->created_at->format('M d, Y H:i:s') }}</div>
                            </div>
                        </li>
                        @if($handoff->queued_at)
                        <li class="d-flex align-items-start mb-3">
                            <i data-feather="clock" class="text-warning me-2 mt-1" style="width:16px;height:16px;"></i>
                            <div>
                                <div class="small fw-semibold">Queued</div>
                                <div class="small text-muted">{{ $handoff->queued_at->format('M d, Y H:i:s') }}</div>
                            </div>
                        </li>
                        @endif
                        @if($handoff->published_at)
                        <li class="d-flex align-items-start mb-3">
                            <i data-feather="check-circle" class="text-success me-2 mt-1" style="width:16px;height:16px;"></i>
                            <div>
                                <div class="small fw-semibold">Published</div>
                                <div class="small text-muted">{{ $handoff->published_at->format('M d, Y H:i:s') }}</div>
                            </div>
                        </li>
                        @endif
                        @if($handoff->withdrawn_at)
                        <li class="d-flex align-items-start mb-3">
                            <i data-feather="x-circle" class="text-secondary me-2 mt-1" style="width:16px;height:16px;"></i>
                            <div>
                                <div class="small fw-semibold">Withdrawn</div>
                                <div class="small text-muted">{{ $handoff->withdrawn_at->format('M d, Y H:i:s') }}</div>
                            </div>
                        </li>
                        @endif
                        @if($handoff->closed_at)
                        <li class="d-flex align-items-start mb-3">
                            <i data-feather="archive" class="text-dark me-2 mt-1" style="width:16px;height:16px;"></i>
                            <div>
                                <div class="small fw-semibold">Closed</div>
                                <div class="small text-muted">{{ $handoff->closed_at->format('M d, Y H:i:s') }}</div>
                            </div>
                        </li>
                        @endif
                    </ul>
                </div>
            </div>

            <!-- Audit Events -->
            <div class="card mb-4">
                <div class="card-header">
                    <h5 class="mb-0">Audit Events</h5>
                </div>
                <div class="card-body p-0">
                    <div style="max-height: 400px; overflow-y: auto;">
                        <table class="table table-sm mb-0" style="font-size: 0.8rem;">
                            <thead>
                                <tr>
                                    <th>Event</th>
                                    <th>By</th>
                                    <th>Result</th>
                                    <th>When</th>
                                </tr>
                            </thead>
                            <tbody>
                                @forelse($handoff->events as $event)
                                    <tr>
                                        <td>
                                            <span class="badge bg-light text-dark">
                                                {{ str_replace('_', ' ', $event->event_type) }}
                                            </span>
                                        </td>
                                        <td>{{ $event->performed_by ?? '—' }}</td>
                                        <td class="text-truncate" style="max-width:120px;" title="{{ $event->result }}">
                                            {{ $event->result ?? '—' }}
                                        </td>
                                        <td>{{ $event->created_at->format('M d H:i') }}</td>
                                    </tr>
                                @empty
                                    <tr>
                                        <td colspan="4" class="text-center text-muted py-3">No events recorded.</td>
                                    </tr>
                                @endforelse
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <!-- External References -->
            @if($handoff->externalRefs->isNotEmpty())
            <div class="card mb-4">
                <div class="card-header">
                    <h5 class="mb-0">External References</h5>
                </div>
                <div class="card-body p-0">
                    <table class="table table-sm mb-0" style="font-size: 0.8rem;">
                        <thead>
                            <tr>
                                <th>Type</th>
                                <th>Value</th>
                                <th>Source</th>
                            </tr>
                        </thead>
                        <tbody>
                            @foreach($handoff->externalRefs as $ref)
                                <tr>
                                    <td><code>{{ $ref->ref_type }}</code></td>
                                    <td class="fw-semibold">{{ $ref->ref_value }}</td>
                                    <td>{{ $ref->ref_source ?? '—' }}</td>
                                </tr>
                            @endforeach
                        </tbody>
                    </table>
                </div>
            </div>
            @endif

            <!-- Sync Errors for this Handoff -->
            @if($handoff->syncErrors->isNotEmpty())
            <div class="card mb-4 border-start border-4 border-warning">
                <div class="card-header">
                    <h5 class="mb-0">Sync Errors</h5>
                </div>
                <div class="card-body p-0">
                    <div style="max-height: 300px; overflow-y: auto;">
                        <table class="table table-sm mb-0" style="font-size: 0.8rem;">
                            <thead>
                                <tr>
                                    <th>Severity</th>
                                    <th>Class</th>
                                    <th>Title</th>
                                    <th>Status</th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                                @foreach($handoff->syncErrors as $err)
                                    <tr class="{{ $err->resolved ? 'opacity-50' : '' }}">
                                        <td>
                                            @php
                                                $sevBadge = match($err->severity) {
                                                    'critical' => 'bg-danger',
                                                    'error'    => 'bg-warning text-dark',
                                                    'warning'  => 'bg-info text-dark',
                                                    default    => 'bg-light text-dark',
                                                };
                                            @endphp
                                            <span class="badge {{ $sevBadge }}">{{ ucfirst($err->severity) }}</span>
                                        </td>
                                        <td><code>{{ $err->error_class }}</code></td>
                                        <td class="text-truncate" style="max-width: 160px;" title="{{ $err->title }}">{{ $err->title }}</td>
                                        <td>
                                            @if($err->resolved)
                                                <span class="badge bg-success">Resolved</span>
                                            @else
                                                <span class="badge bg-danger">Open</span>
                                            @endif
                                        </td>
                                        <td>
                                            @unless($err->resolved)
                                                <form action="{{ route('stloads.sync-error.resolve', $err) }}" method="POST" class="d-inline">
                                                    @csrf
                                                    <button type="submit" class="btn btn-xs btn-outline-success p-1" title="Resolve">
                                                        <i data-feather="check" style="width:12px;height:12px;"></i>
                                                    </button>
                                                </form>
                                            @endunless
                                        </td>
                                    </tr>
                                @endforeach
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
            @endif
        </div>
    </div>
</div>
@endsection

@section('script')
<script>feather.replace();</script>
@endsection
