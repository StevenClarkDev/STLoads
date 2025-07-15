<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Enter OTP - LoadBoard</title>

    <!-- Fonts & Icons -->
    <link href="https://fonts.googleapis.com/css?family=Rubik:400,500,700&display=swap" rel="stylesheet" />
    <link href="https://fonts.googleapis.com/css?family=Roboto:300,400,500,700&display=swap" rel="stylesheet" />
    <link rel="stylesheet" href="../assets/css/vendors/bootstrap.css" />
    <link rel="stylesheet" href="../assets/css/style.css" />
    <link rel="stylesheet" href="../assets/css/color-1.css" />
    <link rel="stylesheet" href="../assets/css/responsive.css" />
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css" />
</head>

<body>
    <div class="container-fluid p-0"
        style="background-image: url('../assets/images/login/texture-bg.jpg'); background-size: cover; background-position: center; min-height: 100vh;">
        <div class="d-flex align-items-center justify-content-center" style="min-height: 100vh;">
            <div class="card p-5 rounded shadow my-4" style="max-width: 500px; width: 100%;">
                <div class="text-center mb-4">
                    <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo"
                        style="max-width: 40%;">
                </div>

                <form action="#" method="POST" class="text-center">
                    <h4>Enter OTP</h4>
                    <p class="text-muted">A 6-digit code has been sent to your email</p>

                    <!-- OTP Inputs -->
                    <div class="d-flex justify-content-between gap-2 my-4">
                        <input type="text" class="form-control text-center" maxlength="1" required style="width: 40px;">
                        <input type="text" class="form-control text-center" maxlength="1" required style="width: 40px;">
                        <input type="text" class="form-control text-center" maxlength="1" required style="width: 40px;">
                        <input type="text" class="form-control text-center" maxlength="1" required style="width: 40px;">
                        <input type="text" class="form-control text-center" maxlength="1" required style="width: 40px;">
                        <input type="text" class="form-control text-center" maxlength="1" required style="width: 40px;">
                    </div>

                    <!-- Verify Button with Timer -->
                    <div id="verify-section">
                        <a href="{{ route('new-password') }}" class="btn btn-primary w-50" id="verifyBtn">
                            Verify <span id="timer" class="ms-2"><i class="fa fa-clock fa-spin"></i> <span
                                    id="time">60</span>s</span>
                        </a>
                    </div>

                    <!-- Resend Button -->
                    <div id="resend-section" class="d-none">
                        <a href="{{ route('otp') }}" class="btn btn-outline-primary w-50 mt-2">Resend OTP</a>
                        <p class="mt-2 text-danger">OTP expired. Click to resend.</p>
                    </div>
                </form>
            </div>
        </div>
    </div>

    <!-- Scripts -->
    <script>
        // Simple countdown
        let seconds = 60;
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

        // Auto move to next input
        const inputs = document.querySelectorAll('input[maxlength="1"]');
        inputs.forEach((input, i) => {
            input.addEventListener("input", () => {
                if (input.value && i < inputs.length - 1) {
                    inputs[i + 1].focus();
                }
            });
        });
    </script>
</body>

</html>