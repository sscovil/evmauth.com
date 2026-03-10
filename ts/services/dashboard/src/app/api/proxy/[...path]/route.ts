import { type NextRequest, NextResponse } from 'next/server';

const BACKEND_URL = process.env.BACKEND_URL ?? 'http://gateway:8000';

async function proxyRequest(request: NextRequest, params: Promise<{ path: string[] }>) {
	const { path } = await params;
	const targetPath = path.join('/');
	const url = `${BACKEND_URL}/${targetPath}`;

	const headers = new Headers();
	headers.set('Content-Type', request.headers.get('Content-Type') ?? 'application/json');
	headers.set('Accept', request.headers.get('Accept') ?? 'application/json');

	// Forward authorization if present
	const authorization = request.headers.get('Authorization');
	if (authorization) {
		headers.set('Authorization', authorization);
	}

	const fetchOptions: RequestInit = {
		method: request.method,
		headers,
	};

	if (request.method !== 'GET' && request.method !== 'HEAD') {
		fetchOptions.body = await request.text();
	}

	const response = await fetch(url, fetchOptions);

	return new NextResponse(response.body, {
		status: response.status,
		statusText: response.statusText,
		headers: {
			'Content-Type': response.headers.get('Content-Type') ?? 'application/json',
		},
	});
}

export async function GET(request: NextRequest, context: { params: Promise<{ path: string[] }> }) {
	return proxyRequest(request, context.params);
}

export async function POST(
	request: NextRequest,
	context: { params: Promise<{ path: string[] }> },
) {
	return proxyRequest(request, context.params);
}

export async function PATCH(
	request: NextRequest,
	context: { params: Promise<{ path: string[] }> },
) {
	return proxyRequest(request, context.params);
}

export async function PUT(request: NextRequest, context: { params: Promise<{ path: string[] }> }) {
	return proxyRequest(request, context.params);
}

export async function DELETE(
	request: NextRequest,
	context: { params: Promise<{ path: string[] }> },
) {
	return proxyRequest(request, context.params);
}
