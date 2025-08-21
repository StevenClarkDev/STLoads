
(function () {
    function ready(fn) { document.readyState !== 'loading' ? fn() : document.addEventListener('DOMContentLoaded', fn); }
    ready(function () {
        const token = document.querySelector('meta[name="csrf-token"]').content;
        axios.defaults.headers.common['X-Requested-With'] = 'XMLHttpRequest';
        axios.defaults.headers.common['X-CSRF-TOKEN'] = token;

        window.Pusher = window.Pusher || Pusher;
        window.Echo = new window.EchoLib.Echo({
            broadcaster: 'pusher',
            key: "{{ config('broadcasting.connections.pusher.key') }}",
            cluster: "{{ config('broadcasting.connections.pusher.options.cluster') }}",
            wsHost: 'ws-' + "{{ config('broadcasting.connections.pusher.options.cluster') }}" + '.pusher.com',
            wsPort: 80, wssPort: 443, forceTLS: true, enabledTransports: ['ws', 'wss'],
            authorizer: (channel) => ({
                authorize: (socketId, cb) => {
                    axios.post('/broadcasting/auth', { socket_id: socketId, channel_name: channel.name })
                        .then(r => cb(false, r.data))
                        .catch(e => cb(true, e));
                }
            })
        });

        const roomId = window.CHAT?.roomId, userId = Number(window.CHAT?.userId);
        if (!roomId) return;

        const messagesEl = document.getElementById('messages');
        const form = document.getElementById('sendForm');
        const input = document.getElementById('msgInput');

        window.Echo.private(`convo.${roomId}`).listen('.message.sent', e => appendMessage(e.message));

        function appendMessage(m) {
            const mine = Number(m.user_id) === userId;
            const el = document.createElement('div');
            el.className = 'msg ' + (mine ? 'right-msg' : 'left-msg');
            el.innerHTML = `
        <div class="msg-bubble">
          <div class="msg-info">
            <div class="msg-info-name">${m.user?.name ?? 'User'}</div>
            <div class="msg-info-time">${new Date(m.created_at ?? Date.now()).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</div>
          </div>
          <div class="msg-text"></div>
        </div>`;
            el.querySelector('.msg-text').textContent = m.body ?? '';
            messagesEl.appendChild(el);
            messagesEl.scrollTop = messagesEl.scrollHeight;
        }

        form?.addEventListener('submit', (e) => {
            e.preventDefault();
            const body = (input.value || '').trim();
            if (!body) return;
            axios.post(`/chat/${roomId}`, { body }).then(({ data }) => {
                // optimistic add (event uses toOthers())
                appendMessage(data.message);
                input.value = '';
            });
        });
    });
})();
