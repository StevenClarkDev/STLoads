@extends('layout.app')

@section('content')
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
                            <div class="header-top">
                                {{-- <a class="btn badge-light-primary f-w-500" href="#!">
                                    <i class="fa fa-plus"></i>
                                </a> --}}
                            </div>
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
                                @endphp
                                <li class="common-space">
                                    <a href="{{ route('chat.room', $c) }}"
                                        class="d-flex justify-content-between align-items-center text-decoration-none w-100
                                            {{ isset($conversation) && $c->id === $conversation->id ? 'bg-light p-2 rounded' : '' }}">
                                        <div class="chat-time">
                                            <div class="active-profile">
                                                <img class="img-fluid rounded-circle"
                                                    src="{{ $other?->avatar_url ?? asset('assets/images/avtar/3.jpg') }}"
                                                    alt="user">
                                                {{-- <div class="status {{ $other?->is_online ?? false ? 'bg-success' : '' }}">
                                                </div> --}}
                                            </div>
                                            <div>
                                                <p>{{ $other?->name ?? 'Unknown user' }}</p>
                                                <p class="text-truncate" style="max-width:180px;">
                                                    {{ $last ? \Illuminate\Support\Str::limit($last->body, 30) : 'No messages yet' }}
                                                </p>
                                            </div>
                                        </div>
                                        <div class="text-end">
                                            <p class="mb-1">{{ $last?->created_at?->diffForHumans() }}</p>
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
                        <div class="common-space">
                            <div class="chat-time">
                                <div class="active-profile">
                                    @php
                                        $other = null;
                                        if ($conversation) {
                                            $other =
                                                \Illuminate\Support\Facades\Auth::id() === $conversation->carrier_id
                                                    ? $conversation->shipper
                                                    : $conversation->carrier;
                                        }
                                    @endphp
                                    <img class="img-fluid rounded-circle"
                                        src="{{ $other?->avatar_url ?? asset('assets/images/blog/comment.jpg') }}"
                                        alt="user">
                                    {{-- <div id="presenceDiv" class="status {{ $other?->is_online ?? false ? 'bg-success' : '' }}">
                                    </div> --}}
                                </div>
                                <div>
                                    <span>{{ $other?->name ?? 'No active conversation' }}</span>
                                    {{-- <p id="presenceText" class="text-muted">Offline</p> --}}
                                    @if ($conversation)
                                        <p class="text-muted">Load #{{ $conversation->load_leg?->leg_code }}</p>
                                        {{-- <small class="text-muted">Load #{{ $conversation->load_leg?->leg_code }}</small> --}}
                                    @endif
                                </div>
                                @if ($conversation)
                                    @php
                                        $iAmCarrier = auth()->id() === ($conversation->carrier_id ?? null);
                                    @endphp

                                    @if ($iAmCarrier && $conversation?->load_leg)
                                        <div>
                                            <div class="d-flex align-items-center gap-2">
                                                <button class="btn btn-primary btn-sm" data-bs-toggle="modal"
                                                    data-bs-target="#offerModal">
                                                    Make Offer
                                                </button>
                                            </div>
                                        </div>
                                    @endif

                                    @php
                                        $iAmOwner =
                                            $conversation &&
                                            auth()->id() === ($conversation->load_leg->load_master->user_id ?? null);
                                        $iAmCarrier = auth()->id() === ($conversation->carrier_id ?? null);
                                    @endphp

                                    @if ($conversation?->load_leg)
                                        <div class="card border mt-2">
                                            <div class="card-header py-2 d-flex justify-content-between align-items-center">
                                                <strong>Offers</strong>
                                                <span id="offersCount" class="badge bg-light text-dark">
                                                    {{ $conversation->load_leg->offers()->count() }} total
                                                </span>
                                            </div>

                                            <div id="offersList" class="card-body py-2">
                                                @forelse($conversation->load_leg->offers()->latest()->get() as $offer)
                                                    <div class="d-flex justify-content-between align-items-center border-bottom py-2"
                                                        data-offer-row="{{ $offer->id }}">
                                                        <div>
                                                            <div class="small text-muted">Carrier
                                                                #{{ $offer->carrier_id }}</div>
                                                            <div>Amount: <strong
                                                                    class="offer-amount">{{ number_format($offer->amount, 2) }}</strong>
                                                            </div>
                                                            <div class="small">
                                                                Status:
                                                                @php
                                                                    $badge =
                                                                        $offer->status_id === 3
                                                                            ? 'bg-success'
                                                                            : ($offer->status_id === 1
                                                                                ? 'bg-warning text-dark'
                                                                                : 'bg-danger');
                                                                    $label =
                                                                        $offer->status_id === 3
                                                                            ? 'Accepted'
                                                                            : ($offer->status_id === 1
                                                                                ? 'Pending'
                                                                                : 'Declined');
                                                                @endphp
                                                                <span class="badge status-badge {{ $badge }}"
                                                                    data-status="{{ $offer->status_id }}">{{ $label }}</span>
                                                            </div>
                                                        </div>

                                                        <div class="offer-actions">
                                                            @if ($iAmOwner && $offer->status_id === 1)
                                                                <button class="btn btn-success btn-sm me-1"
                                                                    data-offer="{{ $offer->id }}"
                                                                    data-action="accept">Accept</button>
                                                                <button class="btn btn-outline-danger btn-sm"
                                                                    data-offer="{{ $offer->id }}"
                                                                    data-action="decline">Decline</button>
                                                            @endif
                                                        </div>
                                                    </div>
                                                @empty
                                                    <div class="text-muted small">No offers yet.</div>
                                                @endforelse

                                                <div id="offerActionMsg" class="small mt-2"></div>
                                            </div>
                                        </div>
                                    @endif
                                @endif
                            </div>

                            {{-- <div class="d-flex gap-2">
                                <div class="contact-edit chat-alert"><i class="icon-info-alt"></i></div>
                                <div class="contact-edit chat-alert">
                                    <svg class="dropdown-toggle" role="menu" data-bs-toggle="dropdown"
                                        aria-expanded="false">
                                        <use href="{{ asset('assets/svg/icon-sprite.svg#menubar') }}"></use>
                                    </svg>
                                    <div class="dropdown-menu dropdown-menu-end">
                                        <a class="dropdown-item" href="#!">View details</a>
                                        <a class="dropdown-item" href="#!">Mute</a>
                                    </div>
                                </div>
                            </div> --}}
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
                                            👋
                                        </div>
                                    @endforelse
                                </div>

                                {{-- SEND FORM --}}
                                <form id="sendForm" class="msger-inputarea"
                                    action="{{ url('/chat/' . $conversation->id) }}" method="POST"
                                    onsubmit="return false;">
                                    @csrf
                                    {{-- <div class="dropdown-form dropdown-toggle" role="main" data-bs-toggle="dropdown"
                                        aria-expanded="false">
                                        <i class="icon-plus"></i>
                                        <div class="chat-icon dropdown-menu dropdown-menu-start">
                                            <div class="dropdown-item mb-2"><svg>
                                                    <use href="{{ asset('assets/svg/icon-sprite.svg#camera') }}"></use>
                                                </svg></div>
                                            <div class="dropdown-item"><svg>
                                                    <use href="{{ asset('assets/svg/icon-sprite.svg#attchment') }}"></use>
                                                </svg></div>
                                        </div>
                                    </div> --}}
                                    <input id="msgInput" name="body" class="msger-input two uk-textarea"
                                        type="text" placeholder="Type Message here.." autocomplete="off">
                                    <div class="open-emoji">
                                        <div id="emojiBtn" class="second-btn uk-button" title="Emoji"></div>
                                    </div>
                                    <emoji-picker id="emojiPicker"
                                        style="display:none; position:absolute; z-index:1000; width:320px; max-height:360px;"></emoji-picker>
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
        <!-- Modal -->
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

            // ✅ Initialize Echo (iife build exposes EchoLib.Echo)
            window.Pusher = window.Pusher || Pusher; // make sure Echo sees Pusher
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
                                console.log('[auth ok]', r
                                    .data); // should print an object with "auth"
                                cb(null, r.data);
                            })
                            .catch(e => {
                                console.log('[auth fail]', e?.response?.status, e?.response
                                    ?.data);
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

            // Position the picker just below the button, within the form
            function positionPicker() {
                if (!emojiBtn || !emojiPicker) return;
                const btnRect = emojiBtn.getBoundingClientRect();
                const formRect = document.getElementById('sendForm').getBoundingClientRect();
                const top = (btnRect.bottom - formRect.top) + 6; // 6px gap
                let left = (btnRect.left - formRect.left);

                // keep it inside the form horizontally
                const maxLeft = formRect.width - emojiPicker.offsetWidth - 8;
                left = Math.max(8, Math.min(left, maxLeft));

                emojiPicker.style.top = `${top}px`;
                emojiPicker.style.left = `${left}px`;
            }

            function togglePicker(show) {
                if (!emojiPicker) return;
                if (show === undefined) show = (emojiPicker.style.display === 'none');
                if (show) {
                    emojiPicker.style.display = 'block';
                    positionPicker();
                } else {
                    emojiPicker.style.display = 'none';
                }
            }

            emojiBtn?.addEventListener('click', (e) => {
                e.preventDefault();
                togglePicker();
            });

            // Insert emoji at cursor position
            function insertAtCursor(el, text) {
                const start = el.selectionStart ?? el.value.length;
                const end = el.selectionEnd ?? el.value.length;
                const before = el.value.slice(0, start);
                const after = el.value.slice(end);
                el.value = before + text + after;
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

            // Close on outside click / Escape
            document.addEventListener('click', (e) => {
                if (!emojiPicker || emojiPicker.style.display === 'none') return;
                if (!emojiPicker.contains(e.target) && !emojiBtn.contains(e.target)) {
                    togglePicker(false);
                }
            });
            document.addEventListener('keydown', (e) => {
                if (e.key === 'Escape') togglePicker(false);
            });

            // Reposition on resize (picker needs to be visible to measure width)
            window.addEventListener('resize', () => {
                if (emojiPicker?.style.display === 'block') positionPicker();
            });

            if (roomId) {
                window.Echo.private(`convo.${roomId}`)
                    .listen('.message.sent', (e) => {
                        console.log('event:', e);
                        appendMessage(e);
                    })
                    .listen('.offer.updated', (e) => {
                        console.log('[offer updated]', e);
                        upsertOfferRow(e.offer);
                        if (Number(e.offer.status_id) === 3) {
                            markOtherPendingsDeclined(e.offer.id);
                        }
                        flashOfferNotice(e.offer);
                    });
            }
            Pusher.logToConsole = true;

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
                        <div class="msg-info-time">${new Date(m.created_at ?? Date.now()).toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'})}</div>
                        </div>
                        <div class="msg-text"></div>
                    </div>`;
                el.querySelector('.msg-text').textContent = m.body ?? '';
                listEl.appendChild(el);
                listEl.scrollTop = listEl.scrollHeight;
            }

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

            // Create or update a single offer row
            function upsertOfferRow(offer) {
                const list = document.getElementById('offersList');
                if (!list) return;

                // update total count if a new offer arrived
                const countBadge = document.getElementById('offersCount');

                let row = list.querySelector(`[data-offer-row="${offer.id}"]`);
                const status = Number(offer.status_id);
                const {
                    badgeClass,
                    label
                } = statusToBadge(status);

                if (!row) {
                    // New row (e.g., when carrier just created an offer)
                    const div = document.createElement('div');
                    div.className = 'd-flex justify-content-between align-items-center border-bottom py-2';
                    div.setAttribute('data-offer-row', offer.id);
                    div.innerHTML = `
      <div>
        <div class="small text-muted">Carrier #${offer.carrier_id}</div>
        <div>Amount: <strong class="offer-amount">${fmtAmount(offer.amount)}</strong></div>
        <div class="small">Status:
          <span class="badge status-badge ${badgeClass}" data-status="${status}">${label}</span>
        </div>
      </div>
      <div class="offer-actions"></div>
    `;
                    list.prepend(div);
                    row = div;

                    // bump count
                    if (countBadge) {
                        const m = countBadge.textContent.match(/\d+/);
                        const curr = m ? Number(m[0]) : 0;
                        countBadge.textContent = `${curr + 1} total`;
                    }
                } else {
                    // Update existing row
                    const amtEl = row.querySelector('.offer-amount');
                    if (amtEl) amtEl.textContent = fmtAmount(offer.amount);

                    const badge = row.querySelector('.status-badge');
                    if (badge) {
                        badge.className = `badge status-badge ${badgeClass}`;
                        badge.textContent = label;
                        badge.setAttribute('data-status', String(status));
                    }
                }

                // Owner-only actions visible ONLY when pending
                const actions = row.querySelector('.offer-actions');
                if (actions) {
                    actions.innerHTML = '';
                    if (status === 1 && !!window.CHAT.isOwner) {
                        actions.innerHTML = `
        <button class="btn btn-success btn-sm me-1" data-offer="${offer.id}" data-action="accept">Accept</button>
        <button class="btn btn-outline-danger btn-sm" data-offer="${offer.id}" data-action="decline">Decline</button>
      `;
                    }
                }
            }

            function markOtherPendingsDeclined(acceptedId) {
                const list = document.getElementById('offersList');
                if (!list) return;
                list.querySelectorAll('[data-offer-row]').forEach(row => {
                    const id = Number(row.getAttribute('data-offer-row'));
                    if (id === Number(acceptedId)) return;
                    const badge = row.querySelector('.status-badge');
                    if (badge && Number(badge.getAttribute('data-status')) === 1) {
                        // flip to declined
                        badge.className = 'badge status-badge bg-danger';
                        badge.textContent = 'Declined';
                        badge.setAttribute('data-status', '0');
                        // remove action buttons since not pending anymore
                        const actions = row.querySelector('.offer-actions');
                        if (actions) actions.innerHTML = '';
                    }
                });
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

            // Small inline flash message (optional)
            function flashOfferNotice(offer) {
                const box = document.getElementById('offerActionMsg');
                if (!box) return;
                const s = Number(offer.status_id);
                box.className = 'small ' + (s === 3 ? 'text-success' : s === 1 ? 'text-muted' : 'text-danger');
                box.textContent =
                    s === 3 ? `Offer #${offer.id} accepted for $${fmtAmount(offer.amount)}` :
                    s === 1 ? `Offer #${offer.id} created for $${fmtAmount(offer.amount)}` :
                    `Offer #${offer.id} declined`;
            }


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
                        console.error('Send failed:', err?.response?.status, err?.response?.data || err);
                        alert(
                            (err?.response?.status === 403) ?
                            'Forbidden: you are not allowed to send in this conversation.' :
                            (err?.response?.status === 419) ?
                            'CSRF token mismatch. Refresh the page and try again.' :
                            'Failed to send message. Check console.'
                        );
                    });
            });

            document.getElementById('chatSearch')?.addEventListener('input', function() {
                const q = this.value.toLowerCase();
                document.querySelectorAll('#convoList li').forEach(li => {
                    li.style.display = li.textContent.toLowerCase().includes(q) ? '' : 'none';
                });
            });

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
                            // optionally close after a short delay
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

            document.addEventListener('click', (e) => {
                const btn = e.target.closest('button[data-offer][data-action]');
                if (!btn) return;
                const id = btn.getAttribute('data-offer');
                const action = btn.getAttribute('data-action'); // accept|decline
                const url = action === 'accept' ?
                    `{{ url('/offers') }}/${id}/accept` :
                    `{{ url('/offers') }}/${id}/decline`;

                axios.post(url)
                    .then(r => {
                        const box = document.getElementById('offerActionMsg');
                        if (box) {
                            box.className = 'text-success small';
                            box.textContent = r.data?.message || 'Done.';
                        }
                    })
                    .catch(err => {
                        const msg = err?.response?.data?.message || 'Action failed.';
                        const box = document.getElementById('offerActionMsg');
                        if (box) {
                            box.className = 'text-danger small';
                            box.textContent = msg;
                        }
                    });
            });
        })();
    </script>
@endpush
