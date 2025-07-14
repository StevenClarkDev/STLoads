<!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="UTF-8" />
  <meta http-equiv="X-UA-Compatible" content="IE=edge" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Create New Password - LoadBoard</title>

  <!-- Google Fonts & Icons -->
  <link href="https://fonts.googleapis.com/css?family=Rubik:400,500,700&display=swap" rel="stylesheet" />
  <link href="https://fonts.googleapis.com/css?family=Roboto:300,400,500,700&display=swap" rel="stylesheet" />
  <link rel="stylesheet" href="../assets/css/vendors/bootstrap.css" />
  <link rel="stylesheet" href="../assets/css/style.css" />
  <link rel="stylesheet" href="../assets/css/color-1.css" />
  <link rel="stylesheet" href="../assets/css/responsive.css" />
  <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css" rel="stylesheet" />
</head>

<body>
  <div class="container-fluid p-0" style="background-image: url('../assets/images/login/texture-bg.jpg'); background-size: cover; background-position: center; min-height: 100vh;">
    <div class="d-flex align-items-center justify-content-center" style="min-height: 100vh;">
      <div class="card p-5 rounded shadow my-4" style="max-width: 500px; width: 100%;">
        <div class="text-center mb-4">
          <img src="../assets/images/stloads/logo-bg_none-small.png" alt="Load Board Logo" style="max-width: 40%;">
        </div>

        <form action="#" method="POST" class="row g-3 text-center">
          <h4>Create Your Password</h4>
          <p class="text-muted">Enter and confirm your new password</p>

          <!-- New Password -->
          <div class="position-relative text-start">
            <label>New Password</label>
            <div class="input-group">
              <input id="new-password" class="form-control pe-5 rounded-2" type="password" name="password" placeholder="********" required>
              <i id="new-password-icon" class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
            </div>
            <small id="password-error" class="text-danger d-none">Password must be at least 8 characters, with a letter and number.</small>
          </div>

          <!-- Confirm Password -->
          <div class="position-relative text-start">
            <label>Confirm Password</label>
            <div class="input-group">
              <input id="confirm-password" class="form-control pe-5 rounded-2" type="password" name="confirm" placeholder="********" required>
              <i id="confirm-password-icon" class="fas fa-check-circle text-muted position-absolute top-50 end-0 translate-middle-y me-3"></i>
            </div>
          </div>

          <!-- Submit -->
          <div class="col-12 mt-3">
            <a href="{{ route('login') }}" class="btn btn-primary w-50">Done</a>
            <p class="mt-3 text-muted">Don’t have an account? <a href="{{ route('register') }}">Create Account</a></p>
          </div>
        </form>
      </div>
    </div>
  </div>

  <!-- JS Logic -->
  <script>
    document.addEventListener("DOMContentLoaded", function () {
      const passwordInput = document.getElementById("new-password");
      const confirmInput = document.getElementById("confirm-password");
      const passIcon = document.getElementById("new-password-icon");
      const confirmIcon = document.getElementById("confirm-password-icon");
      const errorText = document.getElementById("password-error");

      function validatePassword() {
        const value = passwordInput.value;
        const isValid = /^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{8,}$/.test(value);

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
</body>

</html>
