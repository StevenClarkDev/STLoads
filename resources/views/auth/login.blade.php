<!DOCTYPE html>
<html lang="en">

<head>
    <meta http-equiv="Cache-Control" content="no-store, no-cache, must-revalidate">
    <meta http-equiv="Pragma" content="no-cache">
    <meta http-equiv="Expires" content="0">
    <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description"
        content="Cuba admin is super flexible, powerful, clean &amp; modern responsive bootstrap 5 admin template with unlimited possibilities.">
    <meta name="keywords"
        content="admin template, Cuba admin template, dashboard template, flat admin template, responsive admin template, web app">
    <meta name="author" content="pixelstrap">
    <link rel="icon" href="{{ url('assets/images/favicon.png') }}" type="image/x-icon">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css"
        integrity="sha512-b0HZvpK+k...fullhash..." crossorigin="anonymous" referrerpolicy="no-referrer" />
    <link rel="shortcut icon" href="{{ url('assets/images/favicon.png') }}" type="image/x-icon">
    <title>LoadBoard - Login</title>
    <!-- Google font-->
    <link href="https://fonts.googleapis.com/css?family=Rubik:400,400i,500,500i,700,700i&amp;display=swap"
        rel="stylesheet">
    <link href="https://fonts.googleapis.com/css?family=Roboto:300,300i,400,400i,500,500i,700,700i,900&amp;display=swap"
        rel="stylesheet">
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/font-awesome.css') }}">
    <!-- ico-font-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/icofont.css') }}">
    <!-- Themify icon-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/themify.css') }}">
    <!-- Flag icon-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/flag-icon.css') }}">
    <!-- Feather icon-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/feather-icon.css') }}">
    <!-- Plugins css start-->
    <!-- Plugins css Ends-->
    <!-- Bootstrap css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/bootstrap.css') }}">
    <!-- App css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/style.css') }}">
    <link id="color" rel="stylesheet" href="{{ url('assets/css/color-1.css') }}" media="screen">
    <!-- Responsive css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/responsive.css') }}">
</head>

<body>
    <!-- login page start-->
    <div class="container-fluid">
        <div class="row">
            <div class="col-xl-7 p-0">
                <div class="login-card login-dark">
                    <div>
                        <!-- <div><a class="logo text-start" href="index.html"><img class="img-fluid for-light"
                                    src="{{ url('assets/images/logo/logo.png') }}" alt="looginpage"><img
                                    class="img-fluid for-dark" src="{{ url('assets/images/logo/logo_dark.png') }}"
                                    alt="looginpage"></a></div> -->
                        <div class="text-center mb-4">
                            <img src="{{ url('assets/images/stloads/logo-bg_none-small.png') }}" alt="Load Board Logo"
                                style="max-width: 40%;">
                        </div>
                        <div class="login-main">
                            <form class="theme-form" action="{{ url('login') }}" method="post">
                                @csrf
                                <h4>Sign in to account</h4>
                                <p>Enter your email & password to login</p>
                                {{-- Email --}}
                                <div class="form-group position-relative">
                                    <label class="col-form-label">Email Address</label>
                                    <div class="input-group">
                                        <input id="email" class="form-control pe-5 rounded-2" type="email"
                                            name="email" required placeholder="Test@gmail.com">
                                        <input type="hidden" name="id" value="{{ $id ?? '' }}">
                                        <i id="email-icon"
                                            class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                                    </div>
                                </div>
                                {{-- Password --}}
                                <div class="form-group position-relative">
                                    <label class="col-form-label">Password</label>
                                    <div class="form-input position-relative">
                                        <input id="password" class="form-control pe-5" type="password"
                                            name="password" required placeholder="*********">
                                    </div>
                                </div>
                                <div class="form-group" style="margin-bottom: 60px">
                                    <a class="link" href="{{ route('forget-password', $id) }}">Forgot password?</a>
                                </div>
                                <div class="form-group mb-0">
                                    {{-- <div class="checkbox p-0">
                                        <input id="checkbox1" type="checkbox">
                                        <label class="text-muted" for="checkbox1">Remember Me</label>
                                    </div> --}}
                                    <div class="d-flex justify-content-between align-items-center mt-3">
                                        <button class="btn btn-primary btn-block w-40" type="submit">Sign
                                            in</button>
                                        <a href="{{ route('role') }}" class="btn btn-secondary w-40">Back</a>
                                    </div>
                                </div>
                                <!-- <h6 class="text-muted mt-4 or">Or Sign in with</h6>
                                <div class="social mt-4">
                                    <a href="{{ url('/auth/google') }}" target="_blank"
                                        class="btn btn-outline-primary w-100 d-flex align-items-center justify-content-center gap-2">
                                        <i class="fab fa-google me-2"></i> Sign in with Google
                                    </a>
                                </div> -->
                                <p class="mt-4 mb-0 text-center">Don't have account?<a class="ms-2"
                                        href={{ route('register.form', ['id' => $id]) }}>Create Account</a></p>
                            </form>
                        </div>
                    </div>
                </div>
            </div>
            <!-- <div class="col-xl-5"><img class="bg-img-cover bg-center" src="{{ url('assets/images/login/login-poster-2.jpg') }}"
                    alt="loginpage"></div> -->
            <div class="col-xl-5"><img class="bg-img-cover bg-center" src="{{ url('assets/images/login/login-poster.jpg') }}"
                    alt="loginpage"></div>
        </div>
        <!-- latest jquery-->
        <script src="{{ url('assets/js/jquery.min.js') }}"></script>
        <!-- Bootstrap js-->
        <script src="{{ url('assets/js/bootstrap/bootstrap.bundle.min.js') }}"></script>
        <!-- feather icon js-->
        <script src="{{ url('assets/js/icons/feather-icon/feather.min.js') }}"></script>
        <script src="{{ url('assets/js/icons/feather-icon/feather-icon.js') }}"></script>
        <!-- scrollbar js-->
        <!-- Sidebar jquery-->
        <script src="{{ url('assets/js/config.js') }}"></script>
        <!-- Plugins JS start-->
        <!-- Plugins JS Ends-->
        <!-- Theme js-->
        <script src="{{ url('assets/js/script.js') }}"></script>
        <script src="https://cdn.jsdelivr.net/npm/sweetalert2@11"></script>
    </div>
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
</body>


<script>
    document.addEventListener("DOMContentLoaded", function() {
        function validateInput(id, regex, errorId = null) {
            const input = document.getElementById(id);
            const icon = document.getElementById(id + "-icon");
            const error = errorId ? document.getElementById(errorId) : null;
            if (!input || !icon) return;
            input.addEventListener("input", function() {
                const isValid = regex.test(input.value);
                icon.classList.toggle("text-primary", isValid);
                icon.classList.toggle("text-muted", !isValid);
                if (error) {
                    if (input.value.length > 0 && !isValid) {
                        error.classList.remove("d-none");
                    } else {
                        error.classList.add("d-none");
                    }
                }
            });
        }
        validateInput("email", /^[^\s@]+@[^\s@]+\.[^\s@]+$/);
        validateInput("password", /^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{8,}$/, "password-error");
    });
</script>


</html>
