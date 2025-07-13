<!DOCTYPE html>
<html lang="en">

<head>
    <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description"
        content="Cuba admin is super flexible, powerful, clean &amp; modern responsive bootstrap 5 admin template with unlimited possibilities.">
    <meta name="keywords"
        content="admin template, Cuba admin template, dashboard template, flat admin template, responsive admin template, web app">
    <meta name="author" content="pixelstrap">
    <link rel="icon" href="../assets/images/favicon.png" type="image/x-icon">
    <link rel="shortcut icon" href="../assets/images/favicon.png" type="image/x-icon">
    <title>LoadBoard - Signup</title>
     <!-- Google font-->
    <link href="https://fonts.googleapis.com/css?family=Rubik:400,400i,500,500i,700,700i&amp;display=swap"
        rel="stylesheet">
    <link href="https://fonts.googleapis.com/css?family=Roboto:300,300i,400,400i,500,500i,700,700i,900&amp;display=swap"
        rel="stylesheet">
    <link rel="stylesheet" type="text/css" href="../assets/css/font-awesome.css">
    <!-- ico-font-->
    <link rel="stylesheet" type="text/css" href="../assets/css/vendors/icofont.css">
    <!-- Themify icon-->
    <link rel="stylesheet" type="text/css" href="../assets/css/vendors/themify.css">
    <!-- Flag icon-->
    <link rel="stylesheet" type="text/css" href="../assets/css/vendors/flag-icon.css">
    <!-- Feather icon-->
    <link rel="stylesheet" type="text/css" href="../assets/css/vendors/feather-icon.css">
    <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css" rel="stylesheet">
    <!-- Plugins css start-->
    <!-- Plugins css Ends-->
    <!-- Bootstrap css-->
    <link rel="stylesheet" type="text/css" href="../assets/css/vendors/bootstrap.css">
    <!-- App css-->
    <link rel="stylesheet" type="text/css" href="../assets/css/style.css">
    <link id="color" rel="stylesheet" href="../assets/css/color-1.css" media="screen">
    <!-- Responsive css-->
    <link rel="stylesheet" type="text/css" href="../assets/css/responsive.css">
</head>

<body>
    <div class="container-fluid p-0"
        style="background-image: url('../assets/images/login/texture-bg.jpg'); background-size: cover; background-position: center; min-height: 100vh;">
        <div class="d-flex align-items-center justify-content-center" style="min-height: 100vh;">
            <div class="card p-5 rounded shadow my-4" style="max-width: 1100px; width: 100%;">
                <div class="text-center mb-4"> <img src="../assets/images/stloads/logo-bg_none-small.png"
                        alt="Load Board Logo" style="max-width: 30%;"> </div>
                <form class="row g-3" action="{{ route('otp.send') }}" method="POST">
                    @csrf

                    <h4>Sign Up</h4>
                    <p class="text-muted">Enter your details to create an account</p>

                    {{-- Email --}}
                    <div class="col-md-6 position-relative">
                        <label>Email</label>
                        <div class="input-group">
                            <input id="email" class="form-control pe-5 rounded-2" type="email" name="email"
                                placeholder="you@example.com" required>
                            <i id="email-icon"
                                class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                    </div>

                    {{-- Name --}}
                    <div class="col-md-6 position-relative">
                        <label>Name</label>
                        <div class="input-group">
                            <input id="name" class="form-control pe-5 rounded-2" type="text" name="name"
                                placeholder="Full Name" required>
                            <i id="name-icon"
                                class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                    </div>

                    {{-- Password --}}
                    <div class="col-md-6 position-relative">
                        <label>Password</label>
                        <div class="input-group">
                            <input id="password" class="form-control pe-5 rounded-2" type="password" name="password"
                                placeholder="********" required>
                            <i id="password-icon"
                                class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                        <small id="password-error" class="text-danger d-none">Password must be at least 8 characters and
                            contain a letter and number.</small>
                    </div>

                    {{-- Role --}}
                    <div class="col-md-6 position-relative">
                        <label>Role</label>
                        <div class="input-group">
                            <input id="role" class="form-control pe-5 rounded-2" type="text" name="role"
                                placeholder="Carrier / Broker" required>
                        </div>
                    </div>

                    {{-- Confirm Password --}}
                    <div class="col-md-6 position-relative"> <label>Confirm Password</label>
                        <div class="input-group"> <input id="confirm-password" class="form-control pe-5 rounded-2"
                                type="password" name="password_confirmation" placeholder="********" required> <i
                                id="confirm-password-icon"
                                class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                    </div>
                    {{-- DOB + Gender --}}
                    <div class="col-md-6 d-flex gap-2">
                        <div class="flex-fill position-relative">
                            <label>Date of Birth</label>
                            <input class="form-control pe-5 rounded-2 text-muted" type="date" name="dob" required>
                        </div>
                        {{-- Gender Dropdown --}}
                        <div class="flex-fill">
                            <label>Gender</label>
                            <select id="gender" class="form-control pe-5 rounded-2 text-muted" name="gender" required>
                                <option disabled selected>Select Gender</option>
                                <option>Male</option>
                                <option>Female</option>
                                <option>Other</option>
                            </select>
                        </div>

                    </div>

                    {{-- CNIC --}}
                    <div class="col-md-6 position-relative">
                        <label>CNIC</label>
                        <div class="input-group">
                            <input id="cnic" class="form-control pe-5 rounded-2" type="text" name="cnic"
                                placeholder="e.g., 42101XXXXXXXXX" required>
                            <i id="cnic-icon"
                                class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
                        </div>
                    </div>

                    {{-- Address --}}
                    <div class="col-md-6">
                        <label>Address</label>
                        <input class="form-control" type="text" name="address" placeholder="Complete Address" required>
                    </div>

                    {{-- Submit --}}
                    <div class="col-12 text-center mt-4">
                        <button type="submit" class="btn btn-primary w-50">Sign Up</button>
                        <p class="mt-3 text-muted">Already have an account? <a href="{{ route('login') }}">Sign In</a>
                        </p>
                    </div>

                    {{-- Laravel error/success messages --}}
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

    <script>
        document.addEventListener("DOMContentLoaded", function () {
            // Helper function
            function validateInput(id, regex, errorId = null) {
                const input = document.getElementById(id);
                const icon = document.getElementById(id + "-icon");
                const error = errorId ? document.getElementById(errorId) : null;

                if (!input || !icon) return;

                input.addEventListener("input", function () {
                    const isValid = regex.test(input.value);

                    // Tick icon color
                    icon.classList.toggle("text-primary", isValid);
                    icon.classList.toggle("text-muted", !isValid);

                    // Error visibility
                    if (error) {
                        if (input.value.length > 0 && !isValid) {
                            error.classList.remove("d-none");
                        } else {
                            error.classList.add("d-none");
                        }
                    }
                });
            }


            // Validations
            validateInput("email", /^[^\s@]+@[^\s@]+\.[^\s@]+$/);
            validateInput("password", /^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{8,}$/, "password-error");
            validateInput("cnic", /^[0-9]{13}$/);
            validateInput("name", /^[A-Za-z ]{3,}$/);
            const password = document.getElementById("password");
            const confirm = document.getElementById("confirm-password");
            const icon = document.getElementById("confirm-password-icon");

            if (password && confirm && icon) {
                function checkMatch() {
                    const isValid = password.value.trim() === confirm.value.trim() &&
                        confirm.value.length >= 8;
                    icon.classList.toggle("text-primary", isValid);
                    icon.classList.toggle("text-muted", !isValid);
                }

                confirm.addEventListener("input", checkMatch);
                password.addEventListener("input", checkMatch);
            }
        });
        <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js"></script>
</body>

</html>