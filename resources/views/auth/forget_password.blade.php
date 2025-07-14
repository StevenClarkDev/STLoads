<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>LoadBoard - Forgot Password</title>

    <!-- Google fonts -->
    <link href="https://fonts.googleapis.com/css?family=Rubik:400,500,700&display=swap" rel="stylesheet">
    <link href="https://fonts.googleapis.com/css?family=Roboto:300,400,500,700&display=swap" rel="stylesheet">

    <!-- Icons -->
    <link rel="stylesheet" href="../assets/css/font-awesome.css">
    <link rel="stylesheet" href="../assets/css/vendors/icofont.css">
    <link rel="stylesheet" href="../assets/css/vendors/themify.css">
    <link rel="stylesheet" href="../assets/css/vendors/flag-icon.css">
    <link rel="stylesheet" href="../assets/css/vendors/feather-icon.css">
    <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css" rel="stylesheet">

    <!-- Bootstrap + App styles -->
    <link rel="stylesheet" href="../assets/css/vendors/bootstrap.css">
    <link rel="stylesheet" href="../assets/css/style.css">
    <link rel="stylesheet" href="../assets/css/color-1.css" media="screen">
    <link rel="stylesheet" href="../assets/css/responsive.css">
</head>

<body>
    <div class="container-fluid p-0" style="background-image: url('../assets/images/login/texture-bg.jpg'); background-size: cover; background-position: center; min-height: 100vh;">
        <div class="d-flex align-items-center justify-content-center" style="min-height: 100vh;">
            <div class="card p-5 rounded shadow my-4" style="max-width: 500px; width: 100%;">
                <div class="text-center mb-4">
                    <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 40%;">
                </div>

                <form action="{{ route('otp') }}" method="POST" class="row g-3">
                    @csrf

                    <h4>Forgot Password</h4>
                    <p class="text-muted">Enter your email to receive an OTP</p>

                    <!-- Email Field -->
                    <div class="col-12 position-relative">
                        <label>Email</label>
                        <div class="input-group">
                            <input id="email" class="form-control pe-5 rounded-2" type="email" name="email" placeholder="you@example.com" required>
                            <i id="email-icon" class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                    </div>

                    <!-- Submit Button -->
                    <div class="col-12 text-center mt-4">
                        <a href="{{ route('otp') }}" class="btn btn-primary w-50">Send OTP</a>
                        <p class="mt-3 text-muted">Back to <a href="{{ route('login') }}">Sign In</a></p>
                    </div>

                    <!-- Laravel Feedback -->
                    @if ($errors->any())
                        <div class="col-12 text-danger text-center mt-2">
                            {{ $errors->first() }}
                        </div>
                    @endif

                    @if (session('success'))
                        <div class="col-12 text-success text-center mt-2">
                            {{ session('success') }}
                        </div>
                    @endif
                </form>
            </div>
        </div>
    </div>

    <!-- JS -->
    <script>
        document.addEventListener("DOMContentLoaded", function () {
            const emailInput = document.getElementById("email");
            const icon = document.getElementById("email-icon");
            if (emailInput && icon) {
                emailInput.addEventListener("input", function () {
                    const isValid = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(emailInput.value);
                    icon.classList.toggle("text-primary", isValid);
                    icon.classList.toggle("text-muted", !isValid);
                });
            }
        });
    </script>

    <script src="../assets/js/bootstrap/bootstrap.bundle.min.js"></script>
</body>

</html>
