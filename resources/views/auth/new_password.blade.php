@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 500px; width: 100%;">
        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 40%;">
        </div>

        <form action="{{ route('new-password-post', $user->id) }}" method="POST" class="row g-3 text-center">
            @csrf
            <h4>Create Your Password</h4>
            <p class="text-muted">Enter and confirm your new password</p>

            <!-- New Password -->
            <div class="position-relative text-start">
                <label>New Password</label>
                <div class="input-group">
                    <input id="new-password" class="form-control pe-5 rounded-2" type="password" name="password"
                        placeholder="********" required>
                    <i id="new-password-icon"
                        class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                </div>
                <small id="password-error" class="text-danger d-none">Password must be at least 8 characters, with a letter
                    and number.</small>
            </div>

            <!-- Confirm Password -->
            <div class="position-relative text-start">
                <label>Confirm Password</label>
                <div class="input-group">
                    <input id="confirm-password" class="form-control pe-5 rounded-2" type="password" name="password_confirmation"
                        placeholder="********" required>
                    <i id="confirm-password-icon"
                        class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                </div>
            </div>

            <!-- Submit -->
            <div class="col-12 mt-3">
                <button type="submit" class="btn btn-primary w-50">Done</button>
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
            const passwordInput = document.getElementById("new-password");
            const confirmInput = document.getElementById("confirm-password");
            const passIcon = document.getElementById("new-password-icon");
            const confirmIcon = document.getElementById("confirm-password-icon");
            const errorText = document.getElementById("password-error");

            function validatePassword() {
                const value = passwordInput.value;
                const isValid = value.length >= 8;

                passIcon.classList.toggle("text-primary", isValid);
                passIcon.classList.toggle("text-muted", !isValid);
                errorText.classList.toggle("d-none", isValid || value.length === 0);
                validateConfirmPassword(); // Also check confirm match again
            }

            function validateConfirmPassword() {
                const match = passwordInput.value === confirmInput.value && confirmInput.value.length >= 8;
                confirmIcon.classList.toggle("text-primary", match);
                confirmIcon.classList.toggle("text-muted", !match);
            }

            passwordInput.addEventListener("input", validatePassword);
            confirmInput.addEventListener("input", validateConfirmPassword);
        });
    </script>
@endsection
