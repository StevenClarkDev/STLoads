@extends('layout.app')

@section('content')
    <div class="col-xl-9 box-col-6 p-3">
        <div class="card mx-4">
            <div class="card-body">
                <div class="row gy-4 px-4">
                    {{-- <h5 class="mb-3">User Information</h5> --}}
                    <div class="row g-3">
                        <div class="col-sm-12">
                            <strong>Name:</strong>
                            {{ $role->name }}
                        </div>
                        <div class="col-sm-12">
                            <strong>Permissions:</strong>
                            @if (!empty($rolePermissions))
                                @foreach ($rolePermissions as $v)
                                    <label class="form-label">{{ $v->name }},</label>
                                @endforeach
                            @endif
                        </div>
                    </div>
                </div> <!-- end row -->
            </div>
        </div>
    </div>
@endsection
