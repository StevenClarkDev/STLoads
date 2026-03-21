@extends('layout.app')

@section('content')
    <div class="container">

        <div class="d-flex justify-content-between align-items-center mb-3">
            <div>
                <h3 class="mb-0">Tracking Leg #{{ $leg->id }}</h3>
                <small class="text-muted">Load: {{ $leg->leg_code ?? 'N/A' }}</small>
            </div>

            {{-- Status badge --}}
            <div>
                @php
                    $statusName = strtoupper($leg->status_master->name ?? 'Unknown');
                    $statusClass = 'secondary';
                    if (\Illuminate\Support\Str::contains(strtolower($statusName), 'dispatch')) $statusClass = 'info';
                    if (\Illuminate\Support\Str::contains(strtolower($statusName), 'pickup')) $statusClass = 'primary';
                    if (\Illuminate\Support\Str::contains(strtolower($statusName), 'transit')) $statusClass = 'warning';
                    if (\Illuminate\Support\Str::contains(strtolower($statusName), 'delivered')) $statusClass = 'success';
                @endphp
                <span class="badge bg-{{ $statusClass }} px-3 py-2">
                    {{ $statusName }}
                </span>
            </div>
        </div>

        {{-- Tracking indicator + driver actions --}}
        @auth
            @if(auth()->id() === $leg->booked_carrier_id)
                <div class="alert alert-info py-2 mb-3">
                    <div class="d-flex justify-content-between align-items-center">
                        <div>
                            <strong>Driver View:</strong> While this page is open, your location can be used for live tracking.
                        </div>
                        @if(in_array($leg->status_id, [5, 6, 7, 9])) {{-- adjust IDs --}}
                            <span class="badge bg-success" id="tracking-indicator">
                                Tracking: ON
                            </span>
                        @else
                            <span class="badge bg-secondary" id="tracking-indicator">
                                Tracking: OFF
                            </span>
                        @endif
                    </div>
                </div>

                {{-- DRIVER ACTION BUTTONS --}}
                <div class="card mb-3">
                    <div class="card-header">Driver Actions</div>
                    <div class="card-body d-flex flex-wrap gap-2">

                        @if($leg->status_id == 5) {{-- Pickup Started --}}
                            <form method="POST" action="{{ route('leg.pickup.arrived', $leg->id) }}">
                                @csrf
                                <button class="btn btn-primary btn-sm" type="submit">
                                    Arrived at Pickup
                                </button>
                            </form>
                        @endif

                        @if($leg->status_id == 6) {{-- Arrived at Pickup --}}
                            <form method="POST" action="{{ route('leg.pickup.depart', $leg->id) }}">
                                @csrf
                                <button class="btn btn-warning btn-sm" type="submit">
                                    Depart Pickup
                                </button>
                            </form>
                        @endif

                        @if($leg->status_id == 7) {{-- In Transit --}}
                            <form method="POST" action="{{ route('leg.delivery.arrived', $leg->id) }}">
                                @csrf
                                <button class="btn btn-info btn-sm" type="submit">
                                    Arrived at Delivery
                                </button>
                            </form>
                        @endif

                        @if($leg->status_id == 9) {{-- Arrived at Delivery --}}
                            <form method="POST" action="{{ route('leg.delivery.complete', $leg->id) }}">
                                @csrf
                                <button class="btn btn-dark btn-sm" type="submit">
                                    Complete Delivery
                                </button>
                            </form>
                        @endif
                    </div>
                </div>
            @endif
        @endauth

        <div class="row">
            {{-- LEFT COLUMN: Timeline + Documents --}}
            <div class="col-md-5">

                {{-- TIMELINE --}}
                <div class="card mb-3">
                    <div class="card-header d-flex justify-content-between align-items-center">
                        <span>Timeline</span>
                        <small class="text-muted">Most recent at bottom</small>
                    </div>
                    <div class="card-body" style="max-height: 320px; overflow-y: auto;">
                        @if($events->count())
                            <ul class="list-group list-group-flush">
                                @foreach($events as $event)
                                    <li class="list-group-item">
                                        <strong>
                                            {{ ucwords(str_replace('_', ' ', $event->type)) }}
                                        </strong>
                                        <br>
                                        <small class="text-muted">
                                            {{ $event->created_at->format('M d, Y h:i A') }}
                                        </small>
                                    </li>
                                @endforeach
                            </ul>
                        @else
                            <p class="mb-0 text-muted">No timeline events yet.</p>
                        @endif
                    </div>
                </div>

                {{-- DOCUMENTS + UPLOAD --}}
                <div class="card mb-3">
                    <div class="card-header d-flex justify-content-between align-items-center">
                        <span>Documents</span>
                    </div>
                    <div class="card-body">
                        {{-- Existing docs --}}
                        @forelse($documents as $doc)
                            <div class="mb-2">
                                <strong>{{ ucwords(str_replace('_', ' ', $doc->type)) }}</strong><br>
                                <a href="{{ asset('storage/' . $doc->path) }}" target="_blank">
                                    View document
                                </a>
                            </div>
                        @empty
                            <p class="mb-3 text-muted">No documents uploaded yet.</p>
                        @endforelse

                        {{-- Upload form (carrier only, before completion) --}}
                        @auth
                            @if(auth()->id() === $leg->booked_carrier_id && $leg->status_id != 99) {{-- 99 = Completed? adjust --}}
                                <hr>
                                <h6 class="mb-2">Upload Documents</h6>
                                <form action="{{ route('leg.documents.store', $leg->id) }}"
                                      method="POST"
                                      enctype="multipart/form-data"
                                      class="row g-2">
                                    @csrf

                                    <div class="col-12">
                                        <label class="form-label">Document Type</label>
                                        <select name="type" class="form-select" required>
                                            <option value="">Select type</option>
                                            <option value="pickup_bol">Pickup BOL</option>
                                            <option value="pickup_photo">Pickup Photos</option>
                                            <option value="delivery_pod">Delivery POD</option>
                                            <option value="delivery_photo">Delivery Photos</option>
                                            <option value="other">Other</option>
                                        </select>
                                    </div>

                                    <div class="col-12">
                                        <label class="form-label">File</label>
                                        <input type="file" name="file" class="form-control" required>
                                        <small class="text-muted">Max 10 MB. PDF or image recommended.</small>
                                    </div>

                                    <div class="col-12">
                                        <button type="submit" class="btn btn-outline-primary btn-sm">
                                            Upload Document
                                        </button>
                                    </div>
                                </form>
                            @endif
                        @endauth
                    </div>
                </div>

            </div>

            {{-- RIGHT COLUMN: Map --}}
            <div class="col-md-7">
                <div class="card mb-3">
                    <div class="card-header d-flex justify-content-between align-items-center">
                        <span>Live Location</span>
                        @if($lastLocation)
                            <small class="text-muted">
                                Last update: {{ $lastLocation->recorded_at->format('M d, Y h:i A') }}
                            </small>
                        @endif
                    </div>
                    <div class="card-body">
                        @if($lastLocation)
                            <div id="map" style="height: 400px;"></div>
                        @else
                            <p class="mb-1">No location data yet.</p>
                            <small class="text-muted">
                                Once the driver starts pickup and grants GPS permission, you’ll see live updates here.
                            </small>
                        @endif
                    </div>
                </div>
            </div>
        </div>

    </div>
@endsection

@section('scripts')
    {{-- MAP / LEAFLET --}}
    @if($lastLocation)
        <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
        <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>

        <script>
            const map = L.map('map').setView([{{ (float) $lastLocation->lat }}, {{ (float) $lastLocation->lng }}], 10);

            L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                maxZoom: 19,
            }).addTo(map);

            let marker = L.marker([{{ (float) $lastLocation->lat }}, {{ (float) $lastLocation->lng }}]).addTo(map);
            

            // OPTIONAL: Live updating if using Pusher later
            Echo.channel('leg.{{ $leg->id }}.location')
                .listen('.LegLocationUpdated', (e) => {
                    marker.setLatLng([e.lat, e.lng]);
                    map.panTo([e.lat, e.lng]);
                });
        </script>
    @endif

    {{-- GPS TRACKING (CARRIER ONLY) --}}
    @auth
        @if(auth()->id() === $leg->booked_carrier_id && in_array($leg->status_id, [5, 6, 7, 9]))
            <script>
                const Toast = Swal.mixin({
                    toast: true,
                    position: 'top-end',
                    showConfirmButton: false,
                    timer: 3000,
                    timerProgressBar: true,
                });

                if ('geolocation' in navigator) {
                    const legId = {{ $leg->id }};
                    const csrfToken = document.querySelector('meta[name="csrf-token"]').getAttribute('content');

                    let trackingErrorShown = false;
                    let permissionErrorShown = false;

                    const sendPosition = (position) => {
                        const latitude = position.coords.latitude;
                        const longitude = position.coords.longitude;
                        console.log(latitude, longitude);

                        fetch(`/legs/${legId}/location`, {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json',
                                'Accept': 'application/json',
                                'X-CSRF-TOKEN': csrfToken,
                            },
                            body: JSON.stringify({
                                lat: latitude,
                                lng: longitude,
                            }),
                        })
                        .then(res => {
                            if (!res.ok && !trackingErrorShown) {
                                trackingErrorShown = true;
                                return res.json().then(data => {
                                    Toast.fire({
                                        icon: 'error',
                                        title: data.error || "Failed to send location."
                                    });
                                }).catch(() => {
                                    Toast.fire({
                                        icon: 'error',
                                        title: "Failed to send location."
                                    });
                                });
                            }
                        })
                        .catch(() => {
                            if (!trackingErrorShown) {
                                trackingErrorShown = true;
                                Toast.fire({
                                    icon: 'error',
                                    title: "Network error while sending GPS."
                                });
                            }
                        });
                    };

                    const handleError = (error) => {
                        if (!permissionErrorShown) {
                            permissionErrorShown = true;
                            Toast.fire({
                                icon: 'warning',
                                title: 'Please enable location for live tracking.'
                            });
                        }
                    };

                    navigator.geolocation.watchPosition(sendPosition, handleError, {
                        enableHighAccuracy: true,
                        maximumAge: 10000,
                        timeout: 20000,
                    });

                } else {
                    Toast.fire({
                        icon: 'error',
                        title: 'Your device does not support GPS tracking.'
                    });
                }
            </script>
        @endif
    @endauth
@endsection
