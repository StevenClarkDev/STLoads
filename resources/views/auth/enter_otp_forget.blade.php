@extends('auth.app')

@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 500px; width: 100%;">
        <div class="text-center mb-4">
            <img src="{{ asset('assets/images/stloads/logo-bg_none-small.png') }}" alt="Load Board Logo"
                style="max-width: 40%;">
        </div>

        <form action="{{ route('verify-otp-forget') }}" method="POST" class="text-center" id="otpForm">
            @csrf

            <h4>Enter OTP</h4>
            <p class="text-muted">A 6-digit code has been sent to your email</p>

            <div class="d-flex justify-content-between gap-2 my-4">
                <input type="hidden" name="email" value="{{ $to }}">
                @for ($i = 0; $i < 6; $i++)
                    <input type="text" name="otp[]" class="form-control text-center otp-input" maxlength="1" required
                        style="width: 40px;">
                @endfor
            </div>

            <div id="verify-section">
                <button type="submit" class="btn btn-primary w-50" id="verifyBtn">
                    Verify <span id="timer" class="ms-2">
                        <i class="fa fa-clock fa-spin"></i> <span id="time">300</span>s
                    </span>
                </button>
            </div>

            <div id="resend-section" class="d-none">
                <button type="button" class="btn btn-outline-primary w-50 mt-2" id="resendOtpBtn">Resend OTP</button>
                <p class="mt-2 text-danger">OTP expired. Click to resend.</p>
            </div>
        </form>
    </div>
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/sweetalert2@11"></script>
    @if (session('status'))
        <script>
            Swal.fire({
                toast: true,
                position: 'top-end',
                icon: 'success',
                title: {!! json_encode(session('status')) !!},
                showConfirmButton: false,
                timer: 2500
            }).then(() => {
                @php
                    session(['success' => null]);
                @endphp
            });
        </script>
    @endif

    @if ($errors->any())
        <script>
            Swal.fire({
                toast: true,
                position: 'top-end',
                icon: 'error',
                title: {!! json_encode($errors->first()) !!},
                showConfirmButton: false,
                timer: 2500
            });
        </script>
    @endif

    @if (session('error'))
        <script>
            Swal.fire({
                position: 'center',
                icon: 'error',
                title: {!! json_encode(session('error')) !!},
                showConfirmButton: false,
                showCloseButton: true,
                allowOutsideClick: false,
                allowEscapeKey: false,
                backdrop: true,
            }).then(() => {
                @php
                    session(['error' => null]);
                @endphp
            });
        </script>
    @endif

    <script>
        document.addEventListener("DOMContentLoaded", function() {
            let seconds = 300;
            const timer = document.getElementById("time");
            const verifySection = document.getElementById("verify-section");
            const resendSection = document.getElementById("resend-section");

            let countdown = setInterval(updateTimer, 1000);

            function updateTimer() {
                seconds--;
                timer.textContent = seconds;
                if (seconds <= 0) {
                    clearInterval(countdown);
                    verifySection.classList.add("d-none");
                    resendSection.classList.remove("d-none");
                }
            }

            // Auto focus next input
            const inputs = document.querySelectorAll('.otp-input');

            inputs.forEach((input, i) => {
                input.addEventListener("input", (e) => {
                    const value = e.target.value.replace(/\D/g, '');
                    e.target.value = value;

                    if (value && i < inputs.length - 1) {
                        inputs[i + 1].focus();
                    }
                });

                input.addEventListener("keydown", (e) => {
                    if (e.key === "Backspace" && !input.value && i > 0) {
                        inputs[i - 1].focus();
                    }
                });

                input.addEventListener('paste', function(e) {
                    e.preventDefault();
                    const pasteData = (e.clipboardData || window.clipboardData).getData('text');

                    if (/^\d{6}$/.test(pasteData)) {
                        // Fill each input
                        for (let i = 0; i < 6; i++) {
                            inputs[i].value = pasteData[i];
                        }
                        inputs[5].focus(); // move focus to last box
                    }
                });
            });

            // Resend OTP
            document.getElementById("resendOtpBtn").addEventListener("click", function() {
                fetch("{{ route('otp.resend') }}", {
                        method: "POST",
                        headers: {
                            "X-CSRF-TOKEN": "{{ csrf_token() }}",
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            email: "{{ $to }}"
                        }),
                    })
                    .then(response => response.json())
                    .then(data => {
                        if (data.success) {
                            Swal.fire("OTP Sent!", "A new OTP has been sent to your email.", "success");

                            // Reset timer
                            seconds = 300;
                            verifySection.classList.remove("d-none");
                            resendSection.classList.add("d-none");
                            countdown = setInterval(updateTimer, 1000);
                        } else {
                            Swal.fire("Error", data.message || "Unable to resend OTP.", "error");
                        }
                    });
            });
        });
    </script>
@endsection
