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
    <link rel="icon" href="{{ url('assets/images/favicon.png') }}" type="image/x-icon">
    <link rel="shortcut icon" href="{{ url('assets/images/favicon.png') }}" type="image/x-icon">
    <title>LoadBoard - Signup</title>
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
    <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css" rel="stylesheet">
    <!-- Plugins css start-->
    <!-- Plugins css Ends-->
    <!-- Bootstrap css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/vendors/bootstrap.css') }}">
    <!-- App css-->
    <link rel="stylesheet" type="text/css" href="{{ url('assets/css/style.css') }}">
    <link id="color" rel="stylesheet" href="{{ url('assets/css/color-1.css') }}" media="screen">
    <!-- Responsive css-->
    <link rel="stylesheet" type="text/css" href="{{ url('/assets/css/responsive.css') }}">
</head>

<body>
    <div class="container-fluid p-0"
        style="background-image: url('../assets/images/login/texture-bg.jpg'); background-size: cover; background-position: center; min-height: 100vh;">
        <div class="d-flex align-items-center justify-content-center" style="min-height: 100vh;">
            @yield('content')
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