@extends('layout.app')
@section('content')
    <div class="col-xl-9 box-col-6 p-3">
        <div class="card mx-4">
            <div class="card-body">
                <h5 class="mb-3">Role Edit</h5>
                <form class="card-body" method="POST" action="{{ route('roles.update', $role->id) }}">
                    @csrf
                    @method('PUT')
                    <div class="row g-4">
                        <div class="col-md-6">
                            <div class="form-floating form-floating-outline">
                                <input type="text" id="multicol-username" class="form-control" placeholder="Enter Name"
                                    name="name" value="{{ $role->name }}" />
                                <label for="multicol-username">Role</label>
                            </div>
                        </div>
                        @php
                            // Group permissions dynamically based on the first word before the hyphen
                            $groupedPermissions = $permission->groupBy(function ($item) {
                                return explode('-', $item->name)[0]; // Extract the first part before the hyphen
                            });
                        @endphp

                        @foreach ($groupedPermissions as $group => $permissions)
                            <div class="bg-light-primary rounded-2">
                                <h6 class="my-2 ms-2 text-black">{{ ucwords(str_replace('_', ' ', $group)) }}</h6>
                            </div>
                            <div class="row row-bordered g-0">
                                @foreach ($permissions as $value)
                                    <div class="col-md-3 pt-0 p-3">
                                        <div class="form-check mt-3">
                                            <input class="form-check-input" type="checkbox" value="{{ $value->id }}"
                                                id="permission-{{ $value->id }}" name="permission[{{ $value->id }}]"
                                                {{ in_array($value->id, $rolePermissions) ? 'checked' : '' }} />
                                            <label class="form-check-label text-capitalize">
                                                {{-- {{ ucwords(str_replace('-', ' ', $value->name)) }} --}}
                                                {{ ucwords(str_replace('-', ' ', Str::after($value->name, '-'))) }}
                                            </label>
                                        </div>
                                    </div>
                                @endforeach
                            </div>
                        @endforeach
                    </div>
                    <div class="pt-4">
                        <button type="submit" class="btn btn-primary me-sm-3 me-1">Submit</button>
                        <a href="{{ route('roles.index') }}" type="back" class="btn btn-outline-secondary">Cancel</a>
                    </div>
                </form>
            </div>
        </div>
    </div>
@endsection
