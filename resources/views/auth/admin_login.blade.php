@extends('auth.app')
@section('content')
    <div class="card p-5 rounded shadow my-4" style="max-width: 500px; width: 100%;">
        <div class="text-center mb-4">
            <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 40%;">
        </div>

        <form class="theme-form" action="{{ route('admin.login.post') }}" method="post">
            @csrf

            <h4>Sign in to your admin portal</h4>
            <p class="text-muted">Enter your email & password to login</p>

            <!-- Email Field -->
            <div class="col-12 position-relative mb-4">
                <label>Email Address</label>
                <div class="input-group">
                    <input id="email" class="form-control pe-5 rounded-2" type="email" name="email" required
                        placeholder="Test@gmail.com">
                    <input type="hidden" name="id" value="{{ $id ?? '' }}">
                    <i id="email-icon"
                        class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                </div>
            </div>
            <!-- Password Field -->
            <div class="col-12 position-relative">
                <label>Password</label>
                <div class="form-input position-relative">
                    <input id="password" class="form-control pe-5" type="password" name="password" required
                        placeholder="*********">
                    <i class="fas fa-eye pwd-toggle text-muted position-absolute top-50 end-0 translate-middle-y me-3" style="cursor:pointer" title="Show/hide password"></i>
                </div>
            </div>

            <!-- Submit Button -->
            <div class="col-12 text-center mt-4">
                <button class="btn btn-primary btn-sm w-100" type="submit">Sign in</button>
            </div>

        </form>
    </div>

    <!-- Bootstrap JS -->
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
        document.querySelectorAll('.pwd-toggle').forEach(function(icon) {
            icon.addEventListener('click', function() {
                var input = this.parentElement.querySelector('input');
                if (input) {
                    input.type = input.type === 'password' ? 'text' : 'password';
                    this.classList.toggle('fa-eye');
                    this.classList.toggle('fa-eye-slash');
                }
            });
        });
    </script>
@endsection