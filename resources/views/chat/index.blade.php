@extends('layout.app')

@section('content')
    @php
        // Helper: initials for avatar
        function initials($name)
        {
            $p = preg_split('/\s+/', trim($name ?? ''));
            $first = isset($p[0][0]) ? mb_substr($p[0], 0, 1) : '';
            $second = isset($p[1][0]) ? mb_substr($p[1], 0, 1) : (isset($p[0][1]) ? mb_substr($p[0], 1, 1) : '');
            return mb_strtoupper($first . $second);
        }
    @endphp

    <style>
        /* ---------- Avatar (initials fallback) ---------- */
        .avatar-circle {
            width: 42px;
            height: 42px;
            border-radius: 50%;
            display: flex;
            align-items: center;
            justify-content: center;
            background: #e9eef6;
            color: #2b3d63;
            font-weight: 700;
            letter-spacing: .5px;
            box-shadow: inset 0 0 0 1px rgba(0, 0, 0, .06);
        }

        .avatar-circle.sm {
            width: 36px;
            height: 36px;
            font-size: .85rem
        }

        .avatar-circle.lg {
            width: 48px;
            height: 48px;
            font-size: 1rem
        }

        /* ---------- History list in modals ---------- */
        .offer-history {
            max-height: 260px;
            padding: .50rem 0px;
            overflow-y: auto;
        }

        .offer-item {
            display: flex;
            align-items: flex-start;
            justify-content: space-between;
            gap: .75rem;
            padding: .5rem 0;
            border-bottom: 1px dashed #eef1f5;
        }

        .offer-item:last-child {
            border-bottom: 0;
        }

        .offer-when {
            font-size: .825rem;
            color: #6b7280;
            display: flex;
            align-items: center;
            gap: .35rem
        }

        .status-chip {
            border-radius: 999px;
            padding: .15rem .5rem;
            font-size: .75rem;
        }

        /* ---------- Emoji picker polish ---------- */

        /* Put picker above the input bar */
        .msger-inputarea {
            position: relative;
        }

        #emojiPicker {
            display: none;
            position: absolute;
            z-index: 1000;
            bottom: 52px;
            /* sits above the input */
            left: auto;
            right: 54px;
            /* near the emoji button */
            width: 350px;
            max-height: 360px;
        }

        .right-sidebar-title .active-profile img,
        .left-sidebar-chat img {
            object-fit: cover;
        }
    </style>


    <div class="container-fluid">
        <div class="page-title">
            <div class="row">
                <div class="col-6">
                    <h4>Private Chat</h4>
                </div>
                <div class="col-6">
                    <ol class="breadcrumb">
                        <li class="breadcrumb-item">
                            <a href="{{ url('/') }}"><svg class="stroke-icon">
                                    <use href="{{ asset('assets/svg/icon-sprite.svg#stroke-home') }}"></use>
                                </svg></a>
                        </li>
                        <li class="breadcrumb-item">Chat</li>
                        <li class="breadcrumb-item active">Private Chat</li>
                    </ol>
                </div>
            </div>
        </div>
    </div>

    <div class="container-fluid">
        <div class="row g-0 min-vh-100">
            {{-- LEFT: recent chats --}}
            <div class="col-xxl-3 col-xl-4 col-md-5 d-flex flex-column">
                <div class="card left-sidebar-wrapper flex-fill">
                    <div class="left-sidebar-chat">
                        <div class="input-group">
                            <span class="input-group-text"><i class="search-icon text-gray"
                                    data-feather="search"></i></span>
                            <input id="chatSearch" class="form-control" type="text" placeholder="Search here..">
                        </div>
                    </div>

                    <div class="advance-options">
                        <div class="common-space">
                            <p>Recent chats</p>
                            <div class="header-top"></div>
                        </div>

                        {{-- Dynamic conversations list --}}
                        <ul id="convoList" class="chats-user">
                            @forelse($conversations as $c)
                                @php
                                    $other =
                                        \Illuminate\Support\Facades\Auth::id() === $c->carrier_id
                                            ? $c->shipper
                                            : $c->carrier;
                                    $last = $c->latestMessage ?? $c->messages()->latest()->first();
                                    $otherHasAvatar = filled($other?->avatar_url);
                                @endphp
                                <li class="common-space">
                                    <a href="{{ route('chat.room', $c) }}"
                                        class="d-flex justify-content-between align-items-center text-decoration-none w-100
                                                           {{ isset($conversation) && $c->id === $conversation->id ? 'bg-light p-2 rounded' : '' }}">
                                        <div class="chat-time">
                                            <div class="active-profile me-2">
                                                @if ($otherHasAvatar)
                                                    <img class="img-fluid rounded-circle" src="{{ $other->avatar_url }}"
                                                        alt="user" width="36" height="36">
                                                @else
                                                    <div class="avatar-circle sm">{{ initials($other?->name) }}</div>
                                                @endif
                                            </div>
                                            <div>
                                                <p class="mb-0">{{ $other?->name ?? 'Unknown user' }}</p>
                                                <p class="text-truncate text-muted-2 mb-0" style="max-width:180px;">
                                                    {{ $last ? \Illuminate\Support\Str::limit($last->body, 30) : 'No messages yet' }}
                                                </p>
                                            </div>
                                        </div>
                                        <div class="text-end">
                                            <p class="mb-1 text-muted-2">{{ $last?->created_at?->diffForHumans() }}</p>
                                            @if (method_exists($c, 'unread_count_for') && $c->unread_count_for(auth()->user()))
                                                <div class="badge badge-light-success">
                                                    {{ $c->unread_count_for(auth()->user()) }}
                                                </div>
                                            @endif
                                        </div>
                                    </a>
                                </li>
                            @empty
                                <li class="common-space text-muted px-3">No conversations yet.</li>
                            @endforelse
                        </ul>
                    </div>
                </div>
            </div>

            {{-- RIGHT: active conversation --}}
            <div class="col-xxl-9 col-xl-8 col-md-7 d-flex flex-column">
                <div class="card right-sidebar-chat flex-fill d-flex flex-column">
                    <div class="right-sidebar-title">
                        <div class="common-space w-100">
                            <div class="chat-time w-100 d-flex align-items-center justify-content-between">
                                <div class="d-flex align-items-center gap-2">
                                    <div class="active-profile">
                                        @php
                                            $other = null;
                                            if ($conversation) {
                                                $other =
                                                    \Illuminate\Support\Facades\Auth::id() === $conversation->carrier_id
                                                        ? $conversation->shipper
                                                        : $conversation->carrier;
                                            }
                                            $hasAvatar = filled($other?->avatar_url);
                                        @endphp

                                        @if ($hasAvatar)
                                            <img class="img-fluid rounded-circle"
                                                src="{{ $other->avatar_url ?? asset('assets/images/blog/comment.jpg') }}"
                                                alt="user" width="48" height="48">
                                        @else
                                            <div class="avatar-circle lg">{{ initials($other?->name) }}</div>
                                        @endif
                                    </div>
                                    <div class="ms-2">
                                        <span class="fw-semibold">{{ $other?->name ?? 'No active conversation' }}</span>
                                        @if ($conversation)
                                            <p class="text-muted-2 mb-0 small">Load
                                                #{{ $conversation->load_leg?->leg_code }}
                                            </p>
                                        @endif
                                    </div>
                                </div>

                                {{-- Offer action button (single, smart state) --}}
                                @if ($conversation)
                                    @php
                                        $iAmCarrier = auth()->id() === ($conversation->carrier_id ?? null);
                                        $myLatestOffer = null;
                                        if ($iAmCarrier && $conversation?->load_leg) {
                                            $myLatestOffer = $conversation->load_leg
                                                ->offers()
                                                ->where('carrier_id', auth()->id())
                                                ->latest()
                                                ->first();
                                        }
                                        $statusId = $myLatestOffer->status_id ?? null; // 1 pending, 3 accepted, 0/else declined
                                        $approvedAmount =
                                            $statusId === 3 ? number_format($myLatestOffer->amount, 2) : null;
                                    @endphp

                                    <div class="d-flex align-items-center gap-2">
                                        @if ($iAmCarrier && $conversation?->load_leg)
                                            @php
                                                // Decide label & style
                                                if (!$myLatestOffer) {
                                                    $btnLabel = 'Make your first offer';
                                                    $btnClass = 'btn btn-primary';
                                                    $btnData = 'data-offer-action="first"';
                                                } elseif ($statusId === 1) {
                                                    $btnLabel = 'Pending — awaiting shipper';
                                                    $btnClass = 'btn btn-outline-light';
                                                    $btnData = 'data-offer-action="pending"';
                                                } elseif ($statusId === 3) {
                                                    $btnLabel = 'Approved at $' . $approvedAmount;
                                                    $btnClass = 'btn btn-primary';
                                                    $btnData = 'data-offer-action="approved"';
                                                } else {
                                                    $btnLabel = 'Make another offer';
                                                    $btnClass = 'btn btn-outline-primary';
                                                    $btnData = 'data-offer-action="rejected"';
                                                }
                                            @endphp

                                            <button class="{{ $btnClass }}" {!! $btnData !!}>
                                                <span>{{ $btnLabel }}</span>
                                            </button>
                                        @endif
                                        @if ($conversation?->load_leg)
                                            <button class="btn btn-outline-secondary" data-open="offers">
                                                <i class="fa fa-tags me-1"></i> View offers
                                                <span id="offersCountPill"
                                                    class="badge bg-secondary ms-2">{{ $conversation->load_leg->offers()->count() }}</span>
                                            </button>
                                        @endif
                                    </div>
                                @endif
                            </div>

                        </div>
                    </div>

                    @if ($conversation)
                        {{-- Messages + input --}}
                        <div class="right-sidebar-Chats"
                            style="background: url('{{ asset('assets/images/texture.png') }}') repeat; background-size: cover;">
                            <div class="msger">
                                {{-- MESSAGES --}}
                                <div id="messages" class="msger-chat">
                                    @forelse($messages as $m)
                                        <div
                                            class="msg {{ $m->user_id === \Illuminate\Support\Facades\Auth::id() ? 'right-msg' : 'left-msg' }}">
                                            <div class="msg-img"></div>
                                            <div class="msg-bubble">
                                                <div class="msg-info">
                                                    <div class="msg-info-name">{{ $m->user->name }}</div>
                                                    <div class="msg-info-time">{{ $m->created_at->format('h:i A') }}</div>
                                                </div>
                                                <div class="msg-text">{{ $m->body }}</div>
                                            </div>
                                        </div>
                                    @empty
                                        <div class="text-center text-muted py-4">No messages yet — be the first to say hi
                                            👋</div>
                                    @endforelse
                                </div>

                                {{-- SEND FORM --}}
                                <form id="sendForm" class="msger-inputarea"
                                    action="{{ url('/chat/' . $conversation->id) }}" method="POST"
                                    onsubmit="return false;">
                                    @csrf
                                    <input id="msgInput" name="body" class="msger-input two uk-textarea" type="text"
                                        placeholder="Type Message here.." autocomplete="off">
                                    <div class="open-emoji">
                                        <div id="emojiBtn" class="second-btn uk-button" title="Emoji"></div>
                                    </div>
                                    <emoji-picker id="emojiPicker"></emoji-picker>
                                    <button class="msger-send-btn" type="submit"><i
                                            class="fa fa-location-arrow"></i></button>
                                </form>
                            </div>
                        </div>
                    @else
                        {{-- EMPTY STATE --}}
                        <div class="p-5 text-center text-muted">
                            <div class="mb-2">No conversations yet.</div>
                            <div>You can start a chat by submitting a bid from a Load.</div>
                        </div>
                    @endif
                </div>
            </div>
        </div>

        {{-- Modals --}}
        @if ($conversation?->load_leg)
            {{-- 1) Generic "Submit Offer" (used for first/another) --}}
            <div class="modal fade" id="offerModal" tabindex="-1" aria-hidden="true">
                <div class="modal-dialog">
                    <form id="offerForm" class="modal-content" onsubmit="return false;">
                        <div class="modal-header">
                            <h5 class="modal-title">Submit Offer</h5>
                            <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
                        </div>
                        <div class="modal-body">
                            <input type="hidden" name="load_leg_id" value="{{ $conversation->load_leg->id }}">
                            <div class="mb-3">
                                <label class="form-label">Amount</label>
                                <input type="number" min="1" step="0.01" name="amount" class="form-control"
                                    required>
                            </div>

                            {{-- Compact history (3 at a time with scroll) --}}
                            <div class="mb-2">
                                <div class="d-flex align-items-center justify-content-between">
                                    <h6 class="mb-1">Your previous offers</h6>
                                </div>
                                @php
                                    $myOffers = $conversation->load_leg
                                        ->offers()
                                        ->where('carrier_id', auth()->id())
                                        ->latest()
                                        ->get();
                                @endphp
                                <div id="previousOffersList" class="offer-history" style="max-height:210px">
                                    @forelse($myOffers as $o)
                                        @php
                                            $s = (int) $o->status_id;
                                            $chip =
                                                $s === 3
                                                    ? 'bg-success text-white'
                                                    : ($s === 1
                                                        ? 'bg-warning text-dark'
                                                        : 'bg-danger text-white');
                                            $lab = $s === 3 ? 'Approved' : ($s === 1 ? 'Pending' : 'Rejected');
                                        @endphp
                                        <div class="offer-item" data-prev-offer-row="{{ $o->id }}">
                                            <div>
                                                <div class="offer-amount">${{ number_format($o->amount, 2) }}</div>
                                                <div class="offer-when"><i class="fa fa-clock"></i>
                                                    {{ $o->created_at?->format('Y-m-d h:i A') }}</div>
                                            </div>
                                            <span class="status-chip {{ $chip }}"><span
                                                    class="status-text">{{ $lab }}</span></span>
                                        </div>
                                    @empty
                                        <div id="previousOffersEmpty" class="text-muted small">No previous offers.</div>
                                    @endforelse
                                </div>
                            </div>

                            <div id="offerError" class="text-danger small d-none"></div>
                            <div id="offerSuccess" class="text-success small d-none"></div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-light" data-bs-dismiss="modal" type="button">Cancel</button>
                            <button class="btn btn-primary" type="submit">Send Offer</button>
                        </div>
                    </form>
                </div>
            </div>

            {{-- 2) Read-only full history (approved view) --}}
            <div class="modal fade" id="offerHistoryModal" tabindex="-1" aria-hidden="true">
                <div class="modal-dialog modal-dialog-scrollable">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h5 class="modal-title">Your Offers — History</h5>
                            <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
                        </div>
                        <div class="modal-body">
                            @php
                                $myOffersAll = $conversation->load_leg
                                    ->offers()
                                    ->where('carrier_id', auth()->id())
                                    ->orderByDesc('created_at')
                                    ->get();
                            @endphp

                            <div id="offerHistoryList" class="offer-history">
                                @forelse($myOffersAll as $o)
                                    @php
                                        $s = (int) $o->status_id;
                                        $chip =
                                            $s === 3
                                                ? 'bg-success text-white'
                                                : ($s === 1
                                                    ? 'bg-warning text-dark'
                                                    : 'bg-danger text-white');
                                        $lab = $s === 3 ? 'Approved' : ($s === 1 ? 'Pending' : 'Rejected');
                                    @endphp
                                    <div class="offer-item" data-history-row="{{ $o->id }}">
                                        <div>
                                            <div class="offer-amount">${{ number_format($o->amount, 2) }}</div>
                                            <div class="offer-when">
                                                <i class="fa fa-clock"></i>
                                                Offered: <span
                                                    class="created-at">{{ $o->created_at?->format('Y-m-d h:i A') }}</span>
                                            </div>

                                            {{-- this line is shown only when not pending --}}
                                            <div class="offer-when finish-line"
                                                @if ($s === 1) style="display:none" @endif>
                                                <i class="fa fa-flag-checkered"></i>
                                                <span class="final-label">{{ $lab }}</span>:
                                                <span
                                                    class="updated-at">{{ $o->updated_at?->format('Y-m-d h:i A') }}</span>
                                            </div>
                                        </div>
                                        <span class="status-chip {{ $chip }}"><span
                                                class="status-text">{{ $lab }}</span></span>
                                    </div>
                                @empty
                                    <div data-empty="history" class="text-muted small">No offers yet.</div>
                                @endforelse
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-light" data-bs-dismiss="modal">Close</button>
                        </div>
                    </div>
                </div>
            </div>

            {{-- 3) Pending notice (pending view) --}}
            <div class="modal fade" id="offerPendingModal" tabindex="-1" aria-hidden="true">
                <div class="modal-dialog">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h5 class="modal-title">Offer Status</h5>
                            <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
                        </div>
                        <div class="modal-body">
                            <div class="mb-2">
                                <i class="fa fa-hourglass-half me-2"></i>
                                Your latest offer is pending — waiting for shipper’s response.
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-light" data-bs-dismiss="modal">Close</button>
                        </div>
                    </div>
                </div>
            </div>
        @endif
        @if ($conversation?->load_leg)
            @php
                $initialOffers = $conversation->load_leg
                    ->offers()
                    ->latest()
                    ->take(25)
                    ->get(['id', 'amount', 'status_id', 'carrier_id', 'created_at']);
                $isOwner = $conversation && auth()->id() === ($conversation->load_leg->load_master->user_id ?? null);
            @endphp

            <div class="modal fade" id="offersModal" tabindex="-1" aria-hidden="true">
                <div class="modal-dialog modal-dialog-scrollable modal-lg">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h5 class="modal-title">
                                <i class="fa fa-tags me-2"></i>Offers
                                <span class="badge bg-light text-dark ms-2"
                                    id="offersCountModal">{{ $initialOffers->count() }} total</span>
                            </h5>
                            <button type="button" class="btn-close" data-bs-dismiss="modal"></button>
                        </div>

                        <div class="modal-body">
                            <div id="offerActionMsgModal" class="small text-muted mb-2"></div>

                            <div id="offersListModal" class="list-group">
                                @forelse ($initialOffers as $o)
                                    @php
                                        $s = (int) $o->status_id;
                                        $badgeClass =
                                            $s === 3 ? 'bg-success' : ($s === 1 ? 'bg-warning text-dark' : 'bg-danger');
                                        $label = $s === 3 ? 'Accepted' : ($s === 1 ? 'Pending' : 'Declined');
                                    @endphp
                                    <div class="list-group-item d-flex justify-content-between align-items-center"
                                        data-offer-row="{{ $o->id }}">
                                        <div>
                                            <div class="small text-muted">Carrier #{{ $o->carrier_id }}</div>
                                            <div>Amount: <strong
                                                    class="offer-amount">${{ number_format($o->amount, 2) }}</strong>
                                            </div>
                                            <div class="small">Status:
                                                <span class="badge status-badge {{ $badgeClass }}"
                                                    data-status="{{ $s }}">{{ $label }}</span>
                                            </div>
                                        </div>
                                        <div class="offer-actions">
                                            @if ($s === 1 && $isOwner)
                                                <button class="btn btn-success btn-sm me-1"
                                                    data-offer="{{ $o->id }}" data-action="accept">Accept</button>
                                                <button class="btn btn-outline-danger btn-sm"
                                                    data-offer="{{ $o->id }}"
                                                    data-action="decline">Decline</button>
                                            @endif
                                        </div>
                                    </div>
                                @empty
                                    <div id="offerEmpty" class="text-muted small">No offers yet.</div>
                                @endforelse
                            </div>
                        </div>

                        <div class="modal-footer">
                            <button class="btn btn-light" data-bs-dismiss="modal">Close</button>
                        </div>
                    </div>
                </div>
            </div>
        @endif

    </div>

    {{-- expose vars for JS --}}
    <meta name="csrf-token" content="{{ csrf_token() }}">
    <script>
        window.CHAT = {
            roomId: @json($conversation?->id),
            userId: @json(\Illuminate\Support\Facades\Auth::id()),
            isOwner: @json($conversation && auth()->id() === ($conversation->load_leg->load_master->user_id ?? null)),
            pusherKey: "{{ config('broadcasting.connections.pusher.key') }}",
            cluster: "{{ config('broadcasting.connections.pusher.options.cluster') }}"
        };
        console.log('[chat] roomId =', window.CHAT.roomId);
    </script>
@endsection

@push('scripts')
    {{-- Axios + Pusher + Echo (no Vite) --}}
    <script src="{{ url('assets/js/jquery.min.js') }}"></script>
    <script src="https://cdn.jsdelivr.net/npm/axios/dist/axios.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/pusher-js@8/dist/web/pusher.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/laravel-echo@1/dist/echo.iife.js"></script>
    <script type="module" src="https://cdn.jsdelivr.net/npm/emoji-picker-element@^1/index.js"></script>

    <script>
        (function() {
            const token = document.querySelector('meta[name="csrf-token"]').content;
            axios.defaults.headers.common['X-Requested-With'] = 'XMLHttpRequest';
            axios.defaults.headers.common['X-CSRF-TOKEN'] = token;
            axios.defaults.headers.common['Accept'] = 'application/json';

            window.Pusher = window.Pusher || Pusher;
            window.Echo = new Echo({
                broadcaster: 'pusher',
                key: window.CHAT.pusherKey,
                cluster: window.CHAT.cluster,
                forceTLS: true,
                authorizer: (channel) => ({
                    authorize: (socketId, cb) => {
                        axios.post('/broadcasting/auth', {
                                socket_id: socketId,
                                channel_name: channel.name
                            }, {
                                withCredentials: true
                            })
                            .then(r => {
                                cb(null, r.data);
                            })
                            .catch(e => {
                                cb(e, null);
                            });
                    }
                })
            });

            const roomId = window.CHAT.roomId;
            const userId = Number(window.CHAT.userId);
            const listEl = document.getElementById('messages');
            const form = document.getElementById('sendForm');
            const input = document.getElementById('msgInput');
            const emojiBtn = document.getElementById('emojiBtn');
            const emojiPicker = document.getElementById('emojiPicker');

            /* ---------- Emoji picker: open above input, white BG ---------- */
            function togglePicker(show) {
                if (!emojiPicker) return;
                if (show === undefined) show = (emojiPicker.style.display === 'none');
                emojiPicker.style.display = show ? 'block' : 'none';
            }
            emojiBtn?.addEventListener('click', (e) => {
                e.preventDefault();
                togglePicker();
            });

            function insertAtCursor(el, text) {
                const start = el.selectionStart ?? el.value.length;
                const end = el.selectionEnd ?? el.value.length;
                el.value = el.value.slice(0, start) + text + el.value.slice(end);
                const pos = start + text.length;
                el.setSelectionRange(pos, pos);
            }
            emojiPicker?.addEventListener('emoji-click', (ev) => {
                const emoji = ev.detail?.unicode || '';
                if (!emoji) return;
                insertAtCursor(input, emoji);
                input.focus();
                togglePicker(false);
            });
            document.addEventListener('click', (e) => {
                if (!emojiPicker || emojiPicker.style.display === 'none') return;
                if (!emojiPicker.contains(e.target) && !emojiBtn.contains(e.target)) togglePicker(false);
            });
            document.addEventListener('keydown', (e) => {
                if (e.key === 'Escape') togglePicker(false);
            });

            /* ---------- Realtime channel ---------- */
            if (roomId) {
                window.Echo.private(`convo.${roomId}`)
                    .listen('.message.sent', (e) => appendMessage(e))
                    .listen('.offer.updated', (e) => {
                        upsertOfferRowModal(e.offer);
                        upsertOfferHistory(e.offer);
                        upsertPreviousOffer(e.offer);
                        if (Number(e.offer.status_id) === 3) markOtherPendingsDeclinedModal(e.offer.id);
                        flashOfferNoticeModal(e.offer);
                        refreshOfferHeader(e.offer);
                    });
            }

            function refreshOfferHeader(offer) {
                // Only carriers see this smart button in your Blade, so update it for them
                const btn = document.querySelector('[data-offer-action]');
                if (!btn) return;
                const s = Number(offer.status_id);
                const text = (t) => `<span>${t}</span>`;
                if (s === 1) {
                    btn.className = 'btn btn-outline-light';
                    btn.setAttribute('data-offer-action', 'pending');
                    btn.innerHTML = text('Pending — awaiting shipper');
                } else if (s === 3) {
                    btn.className = 'btn btn-primary';
                    btn.setAttribute('data-offer-action', 'approved');
                    btn.innerHTML = text(`Approved at ${fmtAmount(offer.amount)}`);
                } else {
                    btn.className = 'btn btn-outline-primary';
                    btn.setAttribute('data-offer-action', 'rejected');
                    btn.innerHTML = text('Make another offer');
                }
            }
            // Pusher.logToConsole = true;

            function appendMessage(m) {
                if (!listEl) return;
                const mine = Number(m.user_id) === userId;
                const el = document.createElement('div');
                el.className = 'msg ' + (mine ? 'right-msg' : 'left-msg');
                el.innerHTML = `
                            <div class="msg-img"></div>
                            <div class="msg-bubble">
                                <div class="msg-info">
                                    <div class="msg-info-name">${m.user?.name ?? 'User'}</div>
                                    <div class="msg-info-time">${new Date(m.created_at ?? Date.now()).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</div>
                                </div>
                                <div class="msg-text"></div>
                            </div>`;
                el.querySelector('.msg-text').textContent = m.body ?? '';
                listEl.appendChild(el);
                listEl.scrollTop = listEl.scrollHeight;
            }

            // open the offers modal
            document.addEventListener('click', (e) => {
                const btn = e.target.closest('[data-open="offers"]');
                if (!btn) return;
                const m = new bootstrap.Modal(document.getElementById('offersModal'));
                m.show();
            });

            // util
            function fmtAmount(n) {
                try {
                    return new Intl.NumberFormat('en-US', {
                        style: 'currency',
                        currency: 'USD',
                        minimumFractionDigits: 2
                    }).format(Number(n));
                } catch {
                    return Number(n).toFixed(2);
                }
            }

            function statusToBadge(status) {
                if (status === 3) return {
                    badgeClass: 'bg-success',
                    label: 'Accepted'
                };
                if (status === 1) return {
                    badgeClass: 'bg-warning text-dark',
                    label: 'Pending'
                };
                return {
                    badgeClass: 'bg-danger',
                    label: 'Declined'
                };
            }

            // render into the MODAL
            function upsertOfferRowModal(offer) {
                const list = document.getElementById('offersListModal');
                if (!list) return;

                const countBadge = document.getElementById('offersCountModal');
                const pill = document.getElementById('offersCountPill'); // header mini count
                let row = list.querySelector(`[data-offer-row="${offer.id}"]`);
                const status = Number(offer.status_id);
                const {
                    badgeClass,
                    label
                } = statusToBadge(status);

                if (!row) {
                    const div = document.createElement('div');
                    div.className = 'list-group-item d-flex justify-content-between align-items-center';
                    div.dataset.offerRow = offer.id;
                    div.innerHTML = `
      <div>
        <div class="small text-muted">Carrier #${offer.carrier_id}</div>
        <div>Amount: <strong class="offer-amount">${fmtAmount(offer.amount)}</strong></div>
        <div class="small">Status:
          <span class="badge status-badge ${badgeClass}" data-status="${status}">${label}</span>
        </div>
      </div>
      <div class="offer-actions"></div>`;
                    list.prepend(div);
                    row = div;

                    // bump counts
                    if (countBadge) {
                        const m = countBadge.textContent.match(/\d+/);
                        const curr = m ? Number(m[0]) : 0;
                        countBadge.textContent = `${curr + 1} total`;
                    }
                    if (pill) {
                        const curr = Number(pill.textContent || 0);
                        pill.textContent = curr + 1;
                    }
                    document.getElementById('offerEmpty')?.remove();
                } else {
                    row.querySelector('.offer-amount').textContent = fmtAmount(offer.amount);
                    const badge = row.querySelector('.status-badge');
                    badge.className = `badge status-badge ${badgeClass}`;
                    badge.textContent = label;
                    badge.setAttribute('data-status', String(status));
                }

                // owner actions (accept/decline) only when pending
                const actions = row.querySelector('.offer-actions');
                if (actions) {
                    actions.innerHTML = '';
                    if (status === 1 && !!window.CHAT.isOwner) {
                        actions.innerHTML = `
        <button class="btn btn-success btn-sm me-1" data-offer="${offer.id}" data-action="accept">Accept</button>
        <button class="btn btn-outline-danger btn-sm" data-offer="${offer.id}" data-action="decline">Decline</button>`;
                    }
                }
            }

            function markOtherPendingsDeclinedModal(acceptedId) {
                const list = document.getElementById('offersListModal');
                if (!list) return;
                list.querySelectorAll('[data-offer-row]').forEach(row => {
                    const id = Number(row.getAttribute('data-offer-row'));
                    if (id === Number(acceptedId)) return;
                    const badge = row.querySelector('.status-badge');
                    if (badge && Number(badge.getAttribute('data-status')) === 1) {
                        badge.className = 'badge status-badge bg-danger';
                        badge.textContent = 'Declined';
                        badge.setAttribute('data-status', '0');
                        const actions = row.querySelector('.offer-actions');
                        if (actions) actions.innerHTML = '';
                    }
                });
            }

            function flashOfferNoticeModal(offer) {
                const box = document.getElementById('offerActionMsgModal');
                if (!box) return;
                const s = Number(offer.status_id);
                box.className = 'small ' + (s === 3 ? 'text-success' : s === 1 ? 'text-muted' : 'text-danger');
                box.textContent = s === 3 ?
                    `Offer #${offer.id} accepted for ${fmtAmount(offer.amount)}` :
                    s === 1 ?
                    `Offer #${offer.id} created for ${fmtAmount(offer.amount)}` :
                    `Offer #${offer.id} declined`;
            }


            /* ---------- Send message ---------- */
            form?.addEventListener('submit', (e) => {
                e.preventDefault();
                const body = (input?.value || '').trim();
                if (!body) return;
                axios.post(form.getAttribute('action'), {
                        body
                    })
                    .then(() => {
                        input.value = '';
                    })
                    .catch(err => {
                        alert((err?.response?.status === 403) ?
                            'Forbidden: you are not allowed to send in this conversation.' :
                            (err?.response?.status === 419) ?
                            'CSRF token mismatch. Refresh and try again.' :
                            'Failed to send message. Check console.');
                        console.error(err);
                    });
            });

            function upsertOfferHistory(offer) {
                // only show MY offers in the history modal
                if (Number(offer.carrier_id) !== Number(window.CHAT.userId)) return;

                const list = document.getElementById('offerHistoryList');
                if (!list) return;

                const rowSel = `[data-history-row="${offer.id}"]`;
                let row = list.querySelector(rowSel);

                const status = Number(offer.status_id);
                const isPending = status === 1;
                const statusText = status === 3 ? 'Approved' : (isPending ? 'Pending' : 'Rejected');
                const chipClass = status === 3 ? 'bg-success text-white' : (isPending ? 'bg-warning text-dark' :
                    'bg-danger text-white');

                // create row if missing
                if (!row) {
                    row = document.createElement('div');
                    row.className = 'offer-item';
                    row.setAttribute('data-history-row', String(offer.id));
                    row.innerHTML = `
      <div>
        <div class="offer-amount">${fmtAmount(offer.amount)}</div>
        <div class="offer-when">
          <i class="fa fa-clock"></i>
          Offered: <span class="created-at"></span>
        </div>
        <div class="offer-when finish-line" style="${isPending ? 'display:none' : ''}">
          <i class="fa fa-flag-checkered"></i>
          <span class="final-label"></span>:
          <span class="updated-at"></span>
        </div>
      </div>
      <span class="status-chip ${chipClass}"><span class="status-text">${statusText}</span></span>
    `;
                    list.prepend(row);
                    // hide the empty placeholder if it exists
                    list?.querySelector('[data-empty="history"]')?.remove();;
                } else {
                    // update chip
                    const chip = row.querySelector('.status-chip');
                    chip.className = `status-chip ${chipClass}`;
                    chip.querySelector('.status-text').textContent = statusText;
                    // amount might change if you allow counter-offers
                    row.querySelector('.offer-amount').textContent = fmtAmount(offer.amount);
                }

                // set times/labels
                const createdAt = row.querySelector('.created-at');
                const updatedAt = row.querySelector('.updated-at');
                const finalLab = row.querySelector('.final-label');
                if (createdAt && offer.created_at) {
                    createdAt.textContent = new Date(offer.created_at).toLocaleString();
                }
                if (!isPending) {
                    finalLab && (finalLab.textContent = statusText);
                    if (updatedAt && offer.updated_at) {
                        updatedAt.textContent = new Date(offer.updated_at).toLocaleString();
                    }
                    row.querySelector('.finish-line')?.setAttribute('style', ''); // show
                } else {
                    row.querySelector('.finish-line')?.setAttribute('style', 'display:none');
                }
            }

            function upsertPreviousOffer(offer) {
                // Only show MY offers in this modal (carrier’s compact history)
                if (Number(offer.carrier_id) !== Number(window.CHAT.userId)) return;

                const list = document.getElementById('previousOffersList');
                if (!list) return;

                let row = list.querySelector(`[data-prev-offer-row="${offer.id}"]`);
                const s = Number(offer.status_id);
                const label = s === 3 ? 'Approved' : (s === 1 ? 'Pending' : 'Rejected');
                const chip = s === 3 ? 'bg-success text-white' : (s === 1 ? 'bg-warning text-dark' :
                    'bg-danger text-white');

                if (!row) {
                    row = document.createElement('div');
                    row.className = 'offer-item';
                    row.dataset.prevOfferRow = String(offer.id);
                    row.innerHTML = `
      <div>
        <div class="offer-amount">${fmtAmount(offer.amount)}</div>
        <div class="offer-when"><i class="fa fa-clock"></i> ${offer.created_at ? new Date(offer.created_at).toLocaleString() : ''}</div>
      </div>
      <span class="status-chip ${chip}"><span class="status-text">${label}</span></span>`;
                    list.prepend(row);
                    document.getElementById('previousOffersEmpty')?.remove();
                } else {
                    row.querySelector('.offer-amount').textContent = fmtAmount(offer.amount);
                    const chipEl = row.querySelector('.status-chip');
                    chipEl.className = `status-chip ${chip}`;
                    row.querySelector('.status-text').textContent = label;
                }
            }



            /* ---------- Search left list ---------- */
            document.getElementById('chatSearch')?.addEventListener('input', function() {
                const q = this.value.toLowerCase();
                document.querySelectorAll('#convoList li').forEach(li => {
                    li.style.display = li.textContent.toLowerCase().includes(q) ? '' : 'none';
                });
            });

            /* ---------- Offer form submit ---------- */
            const offerForm = document.getElementById('offerForm');
            if (offerForm) {
                offerForm.addEventListener('submit', (e) => {
                    e.preventDefault();
                    const fd = new FormData(offerForm);
                    const payload = Object.fromEntries(fd.entries());
                    axios.post(`{{ route('offers.store') }}`, payload)
                        .then(r => {
                            offerForm.querySelector('#offerError')?.classList.add('d-none');
                            const ok = offerForm.querySelector('#offerSuccess');
                            if (ok) {
                                ok.textContent = r.data?.message ?? 'Offer submitted.';
                                ok.classList.remove('d-none');
                            }
                            setTimeout(() => {
                                const modalEl = document.getElementById('offerModal');
                                if (modalEl) bootstrap.Modal.getInstance(modalEl)?.hide();
                            }, 900);
                        })
                        .catch(err => {
                            const msg = err?.response?.data?.message || 'Failed to send offer.';
                            const el = offerForm.querySelector('#offerError');
                            if (el) {
                                el.textContent = msg;
                                el.classList.remove('d-none');
                            }
                        });
                });
            }

            /* ---------- Owner Accept/Decline ---------- */
            document.addEventListener('click', (e) => {
                const btn = e.target.closest('button[data-offer][data-action]');
                if (!btn) return;
                const id = btn.getAttribute('data-offer');
                const action = btn.getAttribute('data-action');
                const url = action === 'accept' ? `{{ url('/offers') }}/${id}/accept` :
                    `{{ url('/offers') }}/${id}/decline`;
                axios.post(url)
                    .then(r => {
                        ['offerActionMsg', 'offerActionMsgModal'].forEach(id => {
                            const el = document.getElementById(id);
                            if (el) {
                                el.className = 'text-success small';
                                el.textContent = r.data?.message || 'Done.';
                            }
                        });
                    })
                    .catch(err => {
                        const msg = err?.response?.data?.message || 'Action failed.';
                        ['offerActionMsg', 'offerActionMsgModal'].forEach(id => {
                            const el = document.getElementById(id);
                            if (el) {
                                el.className = 'text-danger small';
                                el.textContent = msg;
                            }
                        });
                    });
            });

            /* ---------- Smart Offer Header Button actions ---------- */
            document.addEventListener('click', (e) => {
                const btn = e.target.closest('[data-offer-action]');
                if (!btn) return;
                const kind = btn.getAttribute('data-offer-action');

                if (kind === 'first' || kind === 'rejected') {
                    // open submit modal with compact history (3-at-a-time scroll)
                    const m = new bootstrap.Modal(document.getElementById('offerModal'));
                    m.show();
                    return;
                }
                if (kind === 'approved') {
                    // read-only full history
                    const m = new bootstrap.Modal(document.getElementById('offerHistoryModal'));
                    m.show();
                    return;
                }
                if (kind === 'pending') {
                    const m = new bootstrap.Modal(document.getElementById('offerPendingModal'));
                    m.show();
                    return;
                }
            });
        })();
    </script>
    <script>
        document.getElementById('offerModal')?.addEventListener('show.bs.modal', () => {
            const f = document.getElementById('offerForm');
            f?.reset();
            f?.querySelector('#offerError')?.classList.add('d-none');
            f?.querySelector('#offerSuccess')?.classList.add('d-none');
        });
    </script>
@endpush
