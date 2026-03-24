<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Account Application Update</title>
</head>
<body style="margin:0;padding:0;background-color:#f9f4f4;font-family:Arial,sans-serif;">

    <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#f9f4f4;padding:40px 0;">
        <tr>
            <td align="center">
                <table width="600" cellpadding="0" cellspacing="0" style="background-color:#ffffff;border-radius:12px;overflow:hidden;box-shadow:0 4px 16px rgba(0,0,0,0.08);">

                    {{-- Header --}}
                    <tr>
                        <td style="background-color:#1F537B;padding:40px;text-align:center;">
                            <h1 style="margin:0 0 6px;color:#ffffff;font-size:24px;font-weight:700;">STLoads</h1>
                            <p style="margin:0;color:#a8c8e8;font-size:13px;">Account Application Update</p>
                        </td>
                    </tr>

                    {{-- Body --}}
                    <tr>
                        <td style="padding:40px;">
                            <p style="margin:0 0 16px;color:#1a1a1a;font-size:16px;font-weight:600;">Hello {{ $name }},</p>
                            <p style="margin:0 0 24px;color:#555555;font-size:15px;line-height:1.7;">
                                Thank you for applying to join the STLoads platform as a <strong>{{ $role }}</strong>.
                                After reviewing your application, our compliance team was unable to approve your account at this time.
                            </p>

                            {{-- Remarks box --}}
                            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;">
                                <tr>
                                    <td style="background-color:#fff4f4;border-left:4px solid #e53935;border-radius:4px;padding:16px 20px;">
                                        <p style="margin:0 0 6px;color:#c62828;font-size:13px;font-weight:700;text-transform:uppercase;letter-spacing:0.5px;">Reason provided</p>
                                        <p style="margin:0;color:#555555;font-size:14px;line-height:1.6;">{{ $remarks }}</p>
                                    </td>
                                </tr>
                            </table>

                            <p style="margin:0 0 24px;color:#555555;font-size:15px;line-height:1.7;">
                                If you believe this decision was made in error or if you have questions, please contact our support team by replying to this email.
                            </p>

                            <p style="margin:0;color:#888888;font-size:13px;">
                                Thank you for your interest in STLoads.
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
