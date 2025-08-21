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
                                                <div
                                                    class="status {{ $other?->is_online ?? false ? 'bg-success' : '' }}">
                                                </div>
                                            </div>
                                            <div>
                                                <span>{{ $other?->name ?? 'Unknown user' }}</span>
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
                                    <div class="status {{ $other?->is_online ?? false ? 'bg-success' : '' }}">
                                    </div>
                                </div>
                                <div>
                                    <span>{{ $other?->name ?? 'No active conversation' }}</span>
                                    <p>{{ $other?->is_online ?? false ? 'Online' : 'Offline' }}</p>
                                    @if ($conversation)
                                        <small class="text-muted">Load #{{ $conversation->load_id }}</small>
                                    @endif
                                </div>
                            </div>

                            <div class="d-flex gap-2">
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
                                            👋
                                        </div>
                                    @endforelse
                                </div>

                                {{-- SEND FORM --}}
                                <form id="sendForm" class="msger-inputarea"
                                    action="{{ url('/chat/' . $conversation->id) }}" method="POST"
                                    onsubmit="return false;">
                                    @csrf
                                    <div class="dropdown-form dropdown-toggle" role="main" data-bs-toggle="dropdown"
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
                                    </div>
                                    <input id="msgInput" name="body" class="msger-input two uk-textarea"
                                        type="text" placeholder="Type Message here.." autocomplete="off">
                                    <div class="open-emoji">
                                        <div class="second-btn uk-button"></div>
                                    </div>
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
    </div>
    {{-- expose vars for JS --}}
    <meta name="csrf-token" content="{{ csrf_token() }}">
    <script>
        window.CHAT = {
            roomId: @json($conversation?->id),
            userId: @json(\Illuminate\Support\Facades\Auth::id()),
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
                // enabledTransports: ['ws', 'wss'],
                // (optional) explicit hosts/ports — usually not needed:
                // wsHost: 'ws-' + window.CHAT.cluster + '.pusher.com',
                // wsPort: 80,
                // wssPort: 443,
                authorizer: (channel) => ({
                    authorize: (socketId, cb) => {
                        axios.post('/broadcasting/auth', {
                                socket_id: socketId,
                                channel_name: channel.name
                            })
                            .then(r => cb(false, r.data))
                            .catch(e => cb(true, e));
                    }
                })
            });

            const roomId = window.CHAT.roomId;
            const userId = Number(window.CHAT.userId);
            const listEl = document.getElementById('messages');
            const form = document.getElementById('sendForm');
            const input = document.getElementById('msgInput');

            if (roomId) {
                window.Echo.private(`convo.${roomId}`)
                    .listen('.message.sent', (e) => { console.log('event:', e); appendMessage(e); });
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

            form?.addEventListener('submit', (e) => {
                e.preventDefault();
                const body = (input?.value || '').trim();
                if (!body) return;

                axios.post(form.getAttribute('action'), {
                        body
                    })
                    .then(({
                        data
                    }) => {
                        appendMessage(data.message); // optimistic; broadcast goes to others
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
        })();
    </script>
@endpush
