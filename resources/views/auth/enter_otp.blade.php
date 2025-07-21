@extends('auth.app')

@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 500px; width: 100%;">
        <div class="text-center mb-4">
            <img src="{{ asset('assets/images/stloads/logo-bg_none-small.png') }}" alt="Load Board Logo"
                style="max-width: 40%;">
        </div>

        <form action="{{ route('verify-otp') }}" method="POST" class="text-center">
            @csrf

            <h4>Enter OTP</h4>
            <p class="text-muted">A 6-digit code has been sent to your email</p>

            <div class="d-flex justify-content-between gap-2 my-4">
                <input type="hidden" name="email" value="{{ $to }}">
                {{-- OTP Inputs --}}
                @for ($i = 0; $i < 6; $i++)
                    <input type="text" name="otp[]" class="form-control text-center" maxlength="1" required
                        style="width: 40px;">
                @endfor
            </div>

            <div id="verify-section">
                <button type="submit" class="btn btn-primary w-50" id="verifyBtn">
                    Verify <span id="timer" class="ms-2">
                        <i class="fa fa-clock fa-spin"></i> <span id="time">60</span>s
                    </span>
                </button>
            </div>

            <div id="resend-section" class="d-none">
                {{-- <a href="{{ route('otp.resend') }}" class="btn btn-outline-primary w-50 mt-2">Resend OTP</a> --}}
                <p class="mt-2 text-danger">OTP expired. Click to resend.</p>
            </div>
        </form>
    </div>
@endsection

@section('scripts')
    <script>
        let seconds = 300;
        const timer = document.getElementById("time");
        const verifySection = document.getElementById("verify-section");
        const resendSection = document.getElementById("resend-section");

        const countdown = setInterval(() => {
            seconds--;
            timer.textContent = seconds;
            if (seconds <= 0) {
                clearInterval(countdown);
                verifySection.classList.add("d-none");
                resendSection.classList.remove("d-none");
            }
        }, 1000);

        // Auto focus next input
        const inputs = document.querySelectorAll('input[maxlength="1"]');
        inputs.forEach((input, i) => {
            input.addEventListener("input", () => {
                if (input.value && i < inputs.length - 1) {
                    inputs[i + 1].focus();
                }
            });

            input.addEventListener("keydown", (e) => {
                if (e.key === "Backspace" && !input.value && i > 0) {
                    inputs[i - 1].focus();
                }
            });
        });
    </script>

@endsection
