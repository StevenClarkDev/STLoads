<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Action Required — Account Revision</title>
</head>
<body style="margin:0;padding:0;background-color:#f9f7f3;font-family:Arial,sans-serif;">

    <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#f9f7f3;padding:40px 0;">
        <tr>
            <td align="center">
                <table width="600" cellpadding="0" cellspacing="0" style="background-color:#ffffff;border-radius:12px;overflow:hidden;box-shadow:0 4px 16px rgba(0,0,0,0.08);">

                    {{-- Header --}}
                    <tr>
                        <td style="background-color:#1F537B;padding:40px;text-align:center;">
                            <h1 style="margin:0 0 6px;color:#ffffff;font-size:24px;font-weight:700;">STLoads</h1>
                            <p style="margin:0;color:#a8c8e8;font-size:13px;">Action Required on Your Application</p>
                        </td>
                    </tr>

                    {{-- Body --}}
                    <tr>
                        <td style="padding:40px;">
                            <p style="margin:0 0 8px;color:#1a1a1a;font-size:16px;font-weight:600;">Hello {{ $name }},</p>
                            <p style="margin:0 0 24px;color:#555555;font-size:15px;line-height:1.7;">
                                Our team has reviewed your <strong>{{ $role }}</strong> application and needs a few updates
                                before we can complete the approval process.
                            </p>

                            {{-- Remarks box --}}
                            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;">
                                <tr>
                                    <td style="background-color:#fffbf0;border-left:4px solid #f59e0b;border-radius:4px;padding:16px 20px;">
                                        <p style="margin:0 0 6px;color:#b45309;font-size:13px;font-weight:700;text-transform:uppercase;letter-spacing:0.5px;">⚠️ What needs updating</p>
                                        <p style="margin:0;color:#555555;font-size:14px;line-height:1.6;">{{ $remarks }}</p>
                                    </td>
                                </tr>
                            </table>

                            {{-- CTA --}}
                            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:28px;">
                                <tr>
                                    <td align="center">
                                        <a href="https://portal.stloads.com"
                                           style="display:inline-block;background-color:#1F537B;color:#ffffff;text-decoration:none;font-size:15px;font-weight:700;padding:14px 40px;border-radius:8px;">
                                            Log In &amp; Update My Application →
                                        </a>
                                    </td>
                                </tr>
                            </table>

                            <p style="margin:0;color:#888888;font-size:13px;line-height:1.6;">
                                Once you've made the requested changes, our team will re-review your application promptly.<br>
                                If you have any questions, simply reply to this email.
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
