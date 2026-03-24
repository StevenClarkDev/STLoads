<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Account Approved!</title>
</head>
<body style="margin:0;padding:0;background-color:#f0f7f0;font-family:Arial,sans-serif;">

    <table width="100%" cellpadding="0" cellspacing="0" style="background-color:#f0f7f0;padding:40px 0;">
        <tr>
            <td align="center">
                <table width="600" cellpadding="0" cellspacing="0" style="background-color:#ffffff;border-radius:12px;overflow:hidden;box-shadow:0 4px 16px rgba(0,0,0,0.10);">

                    {{-- Celebration Header --}}
                    <tr>
                        <td style="background:linear-gradient(135deg,#1F537B 0%,#2e7d32 100%);padding:48px 40px 36px;text-align:center;">
                            <div style="font-size:56px;line-height:1;margin-bottom:16px;">🎉</div>
                            <h1 style="margin:0 0 8px;color:#ffffff;font-size:28px;font-weight:800;letter-spacing:0.5px;">You're Approved!</h1>
                            <p style="margin:0;color:#c8e6c9;font-size:15px;">Welcome to the STLoads network</p>
                        </td>
                    </tr>

                    {{-- Confetti bar --}}
                    <tr>
                        <td style="background:linear-gradient(90deg,#f9d35b,#f97c2e,#e84393,#9b59b6,#3498db,#2ecc71);height:5px;"></td>
                    </tr>

                    {{-- Body --}}
                    <tr>
                        <td style="padding:44px 40px 32px;">
                            <p style="margin:0 0 12px;color:#1a1a1a;font-size:18px;font-weight:700;">
                                Congratulations, {{ $name }}! 🙌
                            </p>
                            <p style="margin:0 0 24px;color:#444444;font-size:15px;line-height:1.7;">
                                Your <strong>{{ $role }}</strong> account on STLoads has been <strong style="color:#2e7d32;">verified and approved</strong>
                                by our compliance team. You've cleared all KYC checks and you're ready to roll!
                            </p>

                            {{-- Milestone badge --}}
                            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:32px;">
                                <tr>
                                    <td align="center">
                                        <table cellpadding="0" cellspacing="0">
                                            <tr>
                                                <td style="background-color:#e8f5e9;border:2px solid #66bb6a;border-radius:12px;padding:20px 36px;text-align:center;">
                                                    <div style="font-size:32px;margin-bottom:8px;">✅</div>
                                                    <p style="margin:0;color:#2e7d32;font-size:15px;font-weight:700;">KYC Verified &amp; Account Active</p>
                                                    <p style="margin:4px 0 0;color:#4caf50;font-size:13px;">Approved on {{ $approved_at }}</p>
                                                </td>
                                            </tr>
                                        </table>
                                    </td>
                                </tr>
                            </table>

                            <p style="margin:0 0 24px;color:#444444;font-size:15px;line-height:1.7;">
                                Here's what you can do now:
                            </p>

                            {{-- Next steps --}}
                            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:32px;">
                                <tr>
                                    <td style="background-color:#f8fcff;border-left:4px solid #1F537B;border-radius:4px;padding:16px 20px;">
                                        <p style="margin:0 0 8px;color:#1F537B;font-size:14px;font-weight:700;">🚀 Your next steps</p>
                                        <ul style="margin:0;padding-left:18px;color:#555555;font-size:14px;line-height:2;">
                                            <li>Log in to your account at <strong>portal.stloads.com</strong></li>
                                            <li>Complete your profile and preferences</li>
                                            <li>Start posting or bidding on loads</li>
                                            <li>Connect with partners across the network</li>
                                        </ul>
                                    </td>
                                </tr>
                            </table>

                            {{-- CTA Button --}}
                            <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:32px;">
                                <tr>
                                    <td align="center">
                                        <a href="https://portal.stloads.com"
                                           style="display:inline-block;background-color:#1F537B;color:#ffffff;text-decoration:none;font-size:16px;font-weight:700;padding:16px 48px;border-radius:8px;letter-spacing:0.5px;">
                                            Go to My Dashboard →
                                        </a>
                                    </td>
                                </tr>
                            </table>

                            <p style="margin:0;color:#888888;font-size:13px;line-height:1.6;">
                                Need help getting started? Reply to this email or visit our support page.<br>
                                We're thrilled to have you on board.
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
