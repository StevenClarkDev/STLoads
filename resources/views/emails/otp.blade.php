
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Your OTP Code</title>
</head>
<body style="margin:0;padding:0;background-color:#f4f6f9;font-family:Arial,sans-serif;">

    <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#f4f6f9;padding:40px 0;">
        <tr>
            <td align="center">
                <table width="600" cellpadding="0" cellspacing="0" style="background-color:#ffffff;border-radius:8px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.08);">

                    {{-- Header --}}
                    <tr>
                        <td style="background-color:#1F537B;padding:32px 40px;text-align:center;">
                            <h1 style="margin:0;color:#ffffff;font-size:24px;font-weight:700;letter-spacing:1px;">STLoads</h1>
                            <p style="margin:6px 0 0;color:#a8c8e8;font-size:13px;">Freight &amp; Logistics Platform</p>
                        </td>
                    </tr>

                    {{-- Body --}}
                    <tr>
                        <td style="padding:40px;">
                            <p style="margin:0 0 16px;color:#333333;font-size:16px;">Hello,</p>
                            <p style="margin:0 0 28px;color:#555555;font-size:15px;line-height:1.6;">
                                {{ $context ?? 'Use the code below to complete your registration.' }}
                                This code is valid for <strong>5 minutes</strong>.
                            </p>

                            {{-- OTP Box --}}
                            <table width="100%" cellpadding="0" cellspacing="0">
                                <tr>
                                    <td align="center" style="padding:8px 0 32px;">
                                        <div style="display:inline-block;background-color:#f0f6fc;border:2px dashed #1F537B;border-radius:8px;padding:20px 48px;">
                                            <span style="font-size:42px;font-weight:800;letter-spacing:10px;color:#1F537B;">{{ $otp }}</span>
                                        </div>
                                    </td>
                                </tr>
                            </table>

                            <p style="margin:0 0 8px;color:#888888;font-size:13px;">
                                If you did not request this code, please ignore this email. Do not share this code with anyone.
                            </p>
                        </td>
                    </tr>

                    {{-- Footer --}}
                    <tr>
                        <td style="background-color:#f4f6f9;padding:24px 40px;text-align:center;border-top:1px solid #e8edf2;">
                            <p style="margin:0;color:#aaaaaa;font-size:12px;">&copy; {{ date('Y') }} STLoads. All rights reserved.</p>
                            <p style="margin:6px 0 0;color:#aaaaaa;font-size:12px;">portal.stloads.com</p>
                        </td>
                    </tr>

                </table>
            </td>
        </tr>
    </table>

</body>
</html>
