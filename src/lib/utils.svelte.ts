import { page } from '$app/state';

export function isNotLoginPage() {
    return page.route.id != "/"
}